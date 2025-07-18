use bevy::prelude::*;

use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{mpsc, Arc, Mutex, OnceLock},
};

use chess_core::piece_move::PieceMove;
use thiserror::Error;

use crate::{board::BoardBevy, display::BackgroundColourEvent, game_end::GameEndEvent};

static UCI_TX: OnceLock<Mutex<Option<mpsc::Sender<UciMessage>>>> = OnceLock::new();

#[derive(Error, Debug)]
pub enum UciError {
    #[error("Engine stdin/stdout could not be wrote to/read:\n\t{0}")]
    StdInOutError(#[from] std::io::Error),

    #[error("UCI_TX OnceLock could not be taken out")]
    OnceLockError,

    #[error("UCI_TX Mutex could not be locked")]
    MutexLockError,

    #[error("Could not send message via mpsc using UCI_TX:\n\t{0}")]
    UciTxSendError(#[from] mpsc::SendError<UciMessage>),

    #[error("Could not send message via mpsc using Board_TX:\n\t{0}")]
    BoardTxSendError(#[from] mpsc::SendError<UciToBoardMessage>),

    #[error("Could not find UCI_TX")]
    TxNotFound,

    #[error("Piece move could not be parsed for UCI: {0}")]
    PieceMoveParseError(String),

    #[error("Engine process could not be waited on")]
    EngineProcessWaitError,
}

#[derive(Debug, Clone)]
pub enum UciMessage {
    NewMove { move_history: String },
    CloseChannel,
}

#[derive(Debug, Resource, Clone)]
pub enum UciToBoardMessage {
    BestMove(PieceMove),
}

#[derive(Event, Resource, Debug, Clone)]
pub struct UciToBoardEvent {
    message: UciToBoardMessage,
}
impl UciToBoardEvent {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new(message: UciToBoardMessage) -> Self {
        Self { message }
    }
}

#[derive(Resource)]
pub struct UciToBoardReceiver(pub crossbeam_channel::Receiver<UciToBoardEvent>);

// TODO
/// # Panics
pub fn uci_to_board_event_handler(
    mut ev_uci_to_board: EventReader<UciToBoardEvent>,
    mut commands: Commands,
    mut board: ResMut<BoardBevy>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut transform_query: Query<&mut Transform>,
    mut texture_atlas_query: Query<&mut TextureAtlas>,
    mut game_end_ev: EventWriter<GameEndEvent>,
) {
    // Listen for messages from the Engine Listener thread, then apply moves
    for ev in ev_uci_to_board.read() {
        println!("Board message");

        match ev.message {
            UciToBoardMessage::BestMove(piece_move) => {
                println!(
                    "Applying Move: {piece_move}\t{}",
                    piece_move
                        .to_algebraic()
                        .expect("Could not convert piece move to algebraic in UciToBoard")
                );
                let _ = board.apply_move(
                    &mut commands,
                    &mut transform_query,
                    &mut texture_atlas_query,
                    &mut background_ev,
                    &mut game_end_ev,
                    piece_move,
                );
            }
        }
    }
}

// #[must_use]
// pub fn spawn_uci_to_board_worker() -> UciToBoardReceiverTransmitter {
//     let (tx, rx) = crossbeam_channel::unbounded();

//     // TODO
//     UciToBoardReceiverTransmitter(rx)
// }

#[allow(clippy::needless_pass_by_value)]
pub fn process_uci_to_board_threads(
    tx_rx: Res<UciToBoardReceiver>,
    mut uci_to_board_ev: EventWriter<UciToBoardEvent>,
) {
    for ev in tx_rx.0.try_iter() {
        println!("Event Processing");
        uci_to_board_ev.send(ev);
    }
}

/// # Panics
/// Panics if the engine process cannot start
pub fn communicate_to_uci() -> UciToBoardReceiver {
    // Start the engine process // TODO This will break when the binary is moved
    let mut engine_process = Command::new("target/debug/chess_engine")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start engine");

    let engine_stdin = engine_process.stdin.take().expect("Failed to open stdin");
    let engine_stdout = engine_process.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(engine_stdout);

    // Create a shared stdin
    let shared_stdin = Arc::new(Mutex::new(engine_stdin));

    // This returns when the engine has responded that it is ready for moves
    greet_uci(&shared_stdin, &mut reader).unwrap_or_else(|e| panic!("{e}"));

    // Create a channel to communicate with this process when it is listening to the engine
    let (uci_tx, uci_rx) = std::sync::mpsc::channel();
    UCI_TX
        .set(Mutex::new(Some(uci_tx)))
        .expect("Could not set the UCI_TX OnceLock");

    let (board_tx, board_rx) = crossbeam_channel::unbounded();

    // Create a thread for parsing the UciMessages and sending them to the engine
    let shared_stdin_clone = shared_stdin.clone();
    std::thread::spawn(move || {
        for message in uci_rx {
            board_tx
                .send(UciToBoardEvent::new(UciToBoardMessage::BestMove(
                    PieceMove::from_algebraic("e2e4").unwrap(),
                )))
                .unwrap();

            match_uci_message(
                message,
                &shared_stdin_clone,
                &mut reader,
                &board_tx,
                &mut engine_process,
            )
            .unwrap_or_else(|e| panic!("{e}"));
        }

        println!("Mpsc Channel Closed");
    });

    UciToBoardReceiver(board_rx)
}

/// # Errors
/// Returns an error if the Stdin or Stdout cannot be flushed
pub fn greet_uci(
    stdin: &Arc<Mutex<ChildStdin>>,
    stdout_reader: &mut BufReader<ChildStdout>,
) -> Result<(), UciError> {
    // Initialise the engine
    {
        let mut stdin_locked = stdin.lock().map_err(|_| UciError::MutexLockError)?;
        writeln!(stdin_locked, "uci")?;
        stdin_locked.flush()?;
    }

    // Read and print engine output until it reports "uciok"
    let mut line = String::new();
    loop {
        line.clear();
        stdout_reader.read_line(&mut line)?;
        print!("Engine: {line}");

        // Wait for engine to reply with uciok
        if line.trim() == "uciok" {
            break;
        }
    }

    // Read and print engine output until it reports "readyok"
    uci_is_ready_until_message(stdin, stdout_reader, "readyok")?;

    Ok(())
}

/// # Errors
/// Returns an error if the ``OnceLock`` cannot be got
/// Returns an error if the ``Mutex`` cannot be locked
/// Returns an error if the message cannot be sent
pub fn transmit_to_uci(message: UciMessage) -> Result<(), UciError> {
    UCI_TX
        .get()
        .ok_or(UciError::OnceLockError)?
        .lock()
        .map_err(|_| UciError::MutexLockError)?
        .clone()
        .ok_or(UciError::TxNotFound)?
        .send(message)?;

    Ok(())
}

/// # Errors
/// Returns an error if the ``OnceLock`` cannot be got
/// Returns an error if the ``Mutex`` cannot be locked
pub fn close_channel() -> Result<(), UciError> {
    UCI_TX
        .get()
        .ok_or(UciError::OnceLockError)?
        .lock()
        .map_err(|_| UciError::MutexLockError)?
        .take();

    Ok(())
}

/// # Errors
/// Returns an error if the ``Stdin`` ``Mutex`` cannot be locked
/// Returns an error if ``Stdin`` cannot be flushed
/// Returns an error if a line from ``Stdout`` ``BufReader`` cannot be read
pub fn uci_is_ready_until_message(
    stdin: &Arc<Mutex<ChildStdin>>,
    stdout_reader: &mut BufReader<ChildStdout>,
    message: &str,
) -> Result<(), UciError> {
    // Read and print engine output until it reports "uciok"
    let mut line = String::new();
    loop {
        {
            let mut stdin_locked = stdin.lock().map_err(|_| UciError::MutexLockError)?;
            // Keep sendinig "isready" until "readyok" is given
            writeln!(stdin_locked, "isready")?;
            stdin_locked.flush()?;
        }
        line.clear();
        stdout_reader.read_line(&mut line)?;
        print!("Engine: {line}");

        if line.trim() == message {
            break;
        }
    }

    Ok(())
}

/// # Errors
/// Returns an error if the stdin cant be locked, flushed, or wrote to
/// Returns an error if the stdout reader cannot read a line
/// Returns an error if mpsc channel cannot be closed
/// Returns an error if the engine process cannot be waited on
pub fn match_uci_message(
    message: UciMessage,
    shared_stdin: &Arc<Mutex<ChildStdin>>,
    stdout_reader: &mut BufReader<ChildStdout>,
    board_tx: &crossbeam_channel::Sender<UciToBoardEvent>,
    engine_process: &mut Child,
) -> Result<(), UciError> {
    match message {
        UciMessage::NewMove { move_history } => {
            {
                let mut locked_stdin = shared_stdin.lock().map_err(|_| UciError::MutexLockError)?;

                writeln!(locked_stdin, "position startpos moves {move_history}")?;
            }

            // Read and print engine output until it reports "readyok"
            uci_is_ready_until_message(shared_stdin, stdout_reader, "readyok")?;

            // Tell the engine to find the best move
            {
                let mut locked_stdin = shared_stdin.lock().map_err(|_| UciError::MutexLockError)?;

                writeln!(locked_stdin, "go")?;
            }

            // Read and print engine output until it reports "uciok"
            let mut line = String::new();
            loop {
                line.clear();
                stdout_reader.read_line(&mut line)?;
                print!("Engine: {line}");

                // Wait for engine to reply with a best move
                if line.split_whitespace().next() == Some("bestmove") {
                    break;
                }
            }

            // Best Move was found
            let move_part = line.trim().trim_start_matches("bestmove").trim();

            let piece_move =
                PieceMove::from_algebraic(move_part).map_err(UciError::PieceMoveParseError)?;

            // Send this move to the board
            board_tx
                .send(UciToBoardEvent::new(UciToBoardMessage::BestMove(
                    piece_move,
                )))
                .unwrap();
        }
        UciMessage::CloseChannel => {
            // Close the channel
            close_channel()?;

            // Tell the engine to stop
            {
                let mut locked_stdin = shared_stdin.lock().map_err(|_| UciError::MutexLockError)?;
                // Tell the engine to shutdown
                writeln!(locked_stdin, "quit").unwrap();
                locked_stdin.flush()?;
            }

            // Wait for the process to close
            engine_process
                .wait()
                .map_err(|_| UciError::EngineProcessWaitError)?;
        }
    }

    Ok(())
}
