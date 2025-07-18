use std::fmt;

use bevy::prelude::*;

use chess_core::{
    board::{Board, TilePos, BOARD_SIZE, PLAYERS},
    move_history::HistoryMove,
    piece::Piece,
    piece_move::{
        apply_promotion, handle_castling, handle_en_passant, perform_castling, perform_promotion,
        PieceMove, PieceMoveType,
    },
};

use crate::{
    display::{get_texture_atlas, translate_piece_entity, BackgroundColourEvent},
    game_end::GameEndEvent,
    piece::PieceBundle,
};

#[derive(Resource, Clone, Default)]
pub struct BoardBevy {
    pub board: Board,
    pub entities: [[Option<Entity>; BOARD_SIZE]; BOARD_SIZE],
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
        mut piece_move: PieceMove,
    ) -> (PieceMove, Option<TilePos>, [(bool, bool); 2], Option<Piece>) {
        let mut piece_captured = false;

        // Capture any pieces that should be captured
        let mut piece_moved_to = if self.board.get_piece(piece_move.to) == Piece::None {
            Piece::None
        } else {
            piece_captured = true;
            let captured_entity = self
                .get_entity(piece_move.to)
                .unwrap_or_else(|| panic!("Entity not found at {}", piece_move.to));

            commands.entity(captured_entity).despawn();

            self.board.get_piece(piece_move.to)
        };

        let moved_piece = self.board.get_piece(piece_move.from);

        // Handle promotion
        piece_move = apply_promotion(&mut self.board, moved_piece, piece_move);

        // Update the entity texture to match the promoted piece
        if let PieceMoveType::Promotion(promoted_to) = piece_move.move_type {
            // Change the entity texture to the correct piece
            let piece_entity = self
                .get_entity(piece_move.from)
                .unwrap_or_else(|| panic!("Entity not found for piece at pos {}", piece_move.from));
            let mut texture_atlas = texture_atlas_query
                .get_mut(piece_entity)
                .expect("Could not find piece entity in texture atlas");
            texture_atlas.index = promoted_to.to_bitboard_index();
        }

        // Handle en passant, if this move is en passant, or if this move allows en passant on the next move
        let en_passant_tile;
        (en_passant_tile, piece_move, piece_captured, piece_moved_to) = handle_en_passant(
            &mut self.board,
            piece_move,
            moved_piece,
            piece_captured,
            piece_moved_to,
        )
        .expect("Could not handle en passant in apply_move");

        // Delete the captured piece's  entity
        if piece_move.move_type == PieceMoveType::EnPassant {
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

        // Handle Castling
        let (castling_rights_before_move, kingside_castle);
        (castling_rights_before_move, piece_move, kingside_castle) =
            handle_castling(&mut self.board, piece_move, moved_piece)
                .expect("Castling could not be handled in apply_move");

        // Rook was moved via castling
        if let Some(kingside_castle) = kingside_castle {
            // TODO This is duplicated code
            let (rook_pos, new_rook_pos) = if kingside_castle {
                (
                    TilePos::new(BOARD_SIZE - 1, piece_move.from.rank),
                    TilePos::new(BOARD_SIZE - 3, piece_move.from.rank),
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

        let captured_piece = if piece_captured {
            Some(piece_moved_to)
        } else {
            None
        };

        // Move the piece internally and update its entity translation
        self.move_piece_and_entity(piece_move);
        let piece_entity = self.get_entity(piece_move.to).unwrap_or_else(|| {
            panic!(
                "Moved piece at {} to {}, but no entity was found",
                piece_move.from, piece_move.to
            )
        });
        translate_piece_entity(transform_query, piece_entity, piece_move.to);

        // Check if this move has caused the game to end
        if let Some(winning_player) = self.board.has_game_ended() {
            // Game ended via checkmate or stalemate
            game_end_ev.send(GameEndEvent::new(winning_player));
        } else {
            // Change background colour to show current player
            self.board.next_player();
            background_ev.send(BackgroundColourEvent::new_from_player(
                self.board.get_player(),
            ));
        }

        (
            piece_move,
            en_passant_tile,
            castling_rights_before_move,
            captured_piece,
        )
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
        // Check if this move caused the game to end
        let game_didnt_end = self.board.has_game_ended().is_none();

        let (piece_move, captured_piece, en_passant_tile, castling_rights) = history_move.into();

        // Set the castling rights
        self.board.castling_rights = castling_rights;

        // Set the en_passant marker
        self.board.en_passant_on_last_move = en_passant_tile;

        // Perform the correct move for the move_type
        match piece_move.move_type {
            PieceMoveType::Castling => {
                // Perform the castling
                let moved_piece = self.board.get_piece(piece_move.to);
                let (_, kingside_castle) = perform_castling(
                    &mut self.board,
                    // transform_query,
                    piece_move,
                    moved_piece,
                    true,
                )
                .expect("Castling couldn't be undone");

                // Rook was moved via castling
                if let Some(kingside_castle) = kingside_castle {
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
            }
            PieceMoveType::Promotion(_) => {
                // Get the piece's player as an index
                let player_index = self
                    .board
                    .get_piece(piece_move.to)
                    .to_player()
                    .expect("Player could not be found via piece move for promotion")
                    .to_index();

                // Get this player's pawn type
                let new_piece_type = self
                    .board
                    .get_player_piece(PLAYERS[player_index], Piece::WPawn);

                perform_promotion(
                    &mut self.board,
                    // texture_atlas_query,
                    piece_move.to,
                    new_piece_type,
                );

                // TODO THIS IS DUPLICATED CODE
                // Change the entity texture to the correct piece
                let piece_entity = self.get_entity(piece_move.to).unwrap_or_else(|| {
                    panic!("Entity not found for piece at pos {}", piece_move.to)
                });
                let mut texture_atlas = texture_atlas_query
                    .get_mut(piece_entity)
                    .expect("Could not find piece entity in texture atlas");
                texture_atlas.index = new_piece_type.to_bitboard_index();
            }
            _ => {}
        }

        let piece_entity = self.get_entity(piece_move.to).unwrap_or_else(|| {
            panic!(
                "Entity not found for undo: {}\t\t{:?}\t\t{:?}",
                piece_move.rev(),
                self.get_entity(piece_move.to),
                self.board.move_history.current_idx
            )
        });

        // Move piece before spawning new entities, and also move entity translation
        self.move_piece_and_entity(piece_move.rev());
        translate_piece_entity(transform_query, piece_entity, piece_move.from);

        match piece_move.move_type {
            PieceMoveType::Normal | PieceMoveType::EnPassant => {
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

                    // Update the board to make it aware of the spawned piece
                    self.board.set_piece(captured_piece_tile, captured_piece);
                    self.set_entity(captured_piece_tile, Some(captured_entity.id()));
                }
            }
            _ => {}
        }

        // Only increment the player if the game didn't end on this move
        if game_didnt_end {
            self.board.next_player();
        }

        // Change background colour to show current player
        background_ev.send(BackgroundColourEvent::new_from_player(
            self.board.get_player(),
        ));
    }

    pub fn move_piece_and_entity(&mut self, piece_move: PieceMove) {
        self.board.move_piece(piece_move);

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
        self.entities[tile_pos.file][tile_pos.rank]
    }

    pub const fn set_entity(&mut self, tile_pos: TilePos, entity: Option<Entity>) {
        self.entities[tile_pos.file][tile_pos.rank] = entity;
    }
}
