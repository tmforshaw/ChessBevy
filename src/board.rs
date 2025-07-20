use std::fmt;

use bevy::prelude::*;

use chess_core::{
    board::{Board, TilePos, BOARD_SIZE, PLAYERS},
    move_history::HistoryMove,
    piece::Piece,
    piece_move::{perform_promotion, PieceMove, PieceMoveType},
};

use crate::{
    display::{get_texture_atlas, translate_piece_entity, BackgroundColourEvent},
    game_end::GameEndEvent,
    piece::PieceBundle,
};

#[derive(Resource, Clone, Default)]
pub struct BoardBevy {
    pub board: Board,
    pub entities: [[Option<Entity>; BOARD_SIZE as usize]; BOARD_SIZE as usize],
}

impl std::fmt::Display for BoardBevy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Current Player: {:?}\n{}\n",
            self.board.player, self.board.positions
        )
    }
}

impl BoardBevy {
    /// # Panics
    /// Panics if the piece moved to a tile which isn't ``Piece::None``, but there was no entity found there
    /// Panics if the piece which was moved, but its entity could not be found
    /// Panics if en passant handling fails
    /// Panics if castling handling fails
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::type_complexity)]
    pub fn apply_move(
        &mut self,
        commands: &mut Commands,
        transform_query: &mut Query<&mut Transform>,
        texture_atlas_query: &mut Query<&mut TextureAtlas>,
        background_ev: &mut EventWriter<BackgroundColourEvent>,
        game_end_ev: &mut EventWriter<GameEndEvent>,
        piece_move: PieceMove,
    ) {
        // Capture any pieces that should be captured
        if self.board.positions.get_piece(piece_move.to) != Piece::None {
            if let Some(captured_entity) = self.get_entity(piece_move.to) {
                commands.entity(captured_entity).despawn();
            }
        }

        let move_type = self.board.apply_move(piece_move);

        match move_type {
            PieceMoveType::Normal => {}
            PieceMoveType::EnPassant => {
                // TODO this is duplicated
                // Get the captured piece type from the Board
                let captured_piece_pos = TilePos::new(
                    piece_move.to.file,
                    piece_move.from.rank, // The rank which the piece moved from is the same as the piece it will capture
                );

                // Delete the piece at the captured tile
                let captured_entity = self
                    .get_entity(captured_piece_pos)
                    .expect("Could not get en passant capture entity");
                commands.entity(captured_entity).despawn();
            }
            PieceMoveType::Castling => {
                // Rook was moved via castling
                let kingside_castle = piece_move.from.file > piece_move.to.file;

                // TODO This is duplicated code
                let (rook_pos, new_rook_pos) = if kingside_castle {
                    (
                        TilePos::new(7, piece_move.from.rank),
                        TilePos::new(5, piece_move.from.rank),
                    )
                } else {
                    (
                        TilePos::new(0, piece_move.from.rank),
                        TilePos::new(3, piece_move.from.rank),
                    )
                };

                // TODO This is duplicated code
                // Move the rook entity
                translate_piece_entity(
                    transform_query,
                    self.get_entity(rook_pos)
                        .expect("Rook entity was not at Rook pos"),
                    new_rook_pos,
                );
            }
            PieceMoveType::Promotion(promoted_to) => {
                // Change the entity texture to the correct piece
                let piece_entity = self.get_entity(piece_move.from).unwrap_or_else(|| {
                    panic!("Entity not found for piece at pos {}", piece_move.from)
                });
                let mut texture_atlas = texture_atlas_query
                    .get_mut(piece_entity)
                    .expect("Could not find piece entity in texture atlas");
                texture_atlas.index = promoted_to.to_bitboard_index();
            }
        }

        if let Some(piece_entity) = self.get_entity(piece_move.from) {
            translate_piece_entity(transform_query, piece_entity, piece_move.to);
        }

        // Move the entity internally, after any translations or texture changes are applied
        self.move_entity(piece_move);

        // Check if this move has caused the game to end
        if let Some(winning_player) = self.board.positions.has_game_ended() {
            // Game ended via checkmate or stalemate
            game_end_ev.send(GameEndEvent::new(winning_player));
        } else {
            // self.board.next_player(); // Already performed next player in Board apply_move
            // Change background colour to show current player
            background_ev.send(BackgroundColourEvent::new_from_player(
                self.board.get_player(),
            ));
        }
    }

    /// # Panics
    /// Panics if the move is a promotion and the player cannot be found from the moved piece
    /// Panics if the entity could not be found for the undone piece
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    pub fn undo_move(
        &mut self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        transform_query: &mut Query<&mut Transform>,
        texture_atlas_query: &mut Query<&mut TextureAtlas>,
        background_ev: &mut EventWriter<BackgroundColourEvent>,
        history_move: HistoryMove,
    ) {
        self.board.undo_move(history_move);

        let (piece_move, captured_piece, _, _castling_rights) = history_move.into();

        let piece_entity = self.get_entity(piece_move.to).unwrap_or_else(|| {
            panic!(
                "Entity not found for undo: {}\t\t{:?}\t\t{:?}",
                piece_move.rev(),
                self.get_entity(piece_move.to),
                self.board.move_history.current_idx
            )
        });

        // Move piece before spawning new entities, and also move entity translation
        translate_piece_entity(transform_query, piece_entity, piece_move.from);
        self.move_entity(piece_move.rev());

        match piece_move.move_type {
            PieceMoveType::EnPassant | PieceMoveType::Normal => {
                // Create new entities for any captured pieces
                if let Some(captured_piece) = captured_piece {
                    // Set the captured piece tile, depending on if this capture was an en passant capture or not
                    let captured_piece_tile = if piece_move.move_type == PieceMoveType::EnPassant {
                        TilePos::new(piece_move.to.file, piece_move.from.rank)
                    } else {
                        piece_move.to
                    };

                    let (texture, texture_atlas_layout) =
                        get_texture_atlas(asset_server, texture_atlas_layouts);

                    // Create new entity for the captured piece
                    let captured_entity = commands.spawn(PieceBundle::new(
                        captured_piece_tile.into(),
                        captured_piece,
                        texture,
                        texture_atlas_layout,
                    ));

                    // Update the entities array to make it aware of the spawned piece
                    self.set_entity(captured_piece_tile, Some(captured_entity.id()));
                }
            }
            PieceMoveType::Castling => {
                let kingside_castle = piece_move.from.file > piece_move.to.file;

                // Rook was moved via castling

                // TODO This is duplicated code
                let (rook_pos, new_rook_pos) = if kingside_castle {
                    (
                        TilePos::new(BOARD_SIZE - 1, piece_move.from.rank),
                        TilePos::new(BOARD_SIZE - 1, piece_move.from.rank),
                    )
                } else {
                    (
                        TilePos::new(0, piece_move.from.rank),
                        TilePos::new(0, piece_move.from.rank),
                    )
                };

                // TODO This is duplicated code
                // Move the rook entity
                translate_piece_entity(
                    transform_query,
                    self.get_entity(rook_pos)
                        .expect("Rook entity was not at Rook pos"),
                    new_rook_pos,
                );

                // TODO This is duplicated code
                // Move the rook (and its entity ID) internally
                self.move_entity(PieceMove::new(rook_pos, new_rook_pos));
            }
            PieceMoveType::Promotion(promoted_to) => {
                // Get the piece's player as an index
                let player_index = promoted_to
                    .to_player()
                    .expect("Player could not be found via piece move for promotion")
                    .to_index();

                // Get this player's pawn type
                let player_pawn = Piece::get_player_piece(PLAYERS[player_index], Piece::WPawn);

                // TODO THIS IS DUPLICATED CODE
                // Change the entity texture to the correct piece
                let piece_entity = self.get_entity(piece_move.from).unwrap_or_else(|| {
                    panic!("Entity not found for piece at pos {}", piece_move.from)
                });
                let mut texture_atlas = texture_atlas_query
                    .get_mut(piece_entity)
                    .expect("Could not find piece entity in texture atlas");
                texture_atlas.index = player_pawn.to_bitboard_index();
            }
        }

        // Change background colour to show current player
        background_ev.send(BackgroundColourEvent::new_from_player(
            self.board.get_player(),
        ));
    }

    pub fn move_piece_and_entity(&mut self, piece_move: PieceMove) {
        self.board.positions.move_piece(piece_move);

        let moved_entity = self.get_entity(piece_move.from);
        self.set_entity(piece_move.from, None);
        self.set_entity(piece_move.to, moved_entity);
    }

    pub const fn move_entity(&mut self, piece_move: PieceMove) {
        let moved_entity = self.get_entity(piece_move.from);
        self.set_entity(piece_move.from, None);
        self.set_entity(piece_move.to, moved_entity);
    }

    #[must_use]
    pub const fn get_entity(&self, tile_pos: TilePos) -> Option<Entity> {
        self.entities[tile_pos.file as usize][tile_pos.rank as usize]
    }

    pub const fn set_entity(&mut self, tile_pos: TilePos, entity: Option<Entity>) {
        self.entities[tile_pos.file as usize][tile_pos.rank as usize] = entity;
    }
}
