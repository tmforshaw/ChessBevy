use bevy::prelude::*;
use thiserror::Error;

use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{mpsc, Arc, Mutex, OnceLock},
};

use chess_core::{
    board::{Player, DEFAULT_FEN},
    piece_move::PieceMove,
};

use crate::{
    uci_event::{UciToBoardMessage, UciToBoardReceiver},
    uci_info::send_uci_info,
};

const ENGINE_COMMAND: &str = "stockfish";
// const ENGINE_COMMAND: &str = "target/debug/chess_engine";

pub const ENGINE_PLAYER: Player = Player::Black;

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
    BoardTxSendError(#[from] crossbeam_channel::SendError<UciToBoardMessage>),

    #[error("Could not find UCI_TX")]
    TxNotFound,

    #[error("Piece move could not be parsed for UCI:\n\t{0}")]
    PieceMoveParseError(String),

    #[error("Engine process could not be waited on")]
    EngineProcessWaitError,

    #[error("Integer couldn't be parsed:\n\t{0}")]
    NumericalParseError(#[from] std::num::ParseIntError),
}

#[derive(Debug, Clone)]
pub enum UciMessage {
    NewMove { move_history: String, player_to_move: Player },
    UpdateEval { move_history: String, player_to_move: Player },
    CloseChannel,
}

/// # Panics
/// Panics if the engine process cannot start
pub fn communicate_to_uci() -> UciToBoardReceiver {
    // Start the engine process // TODO This will break when the binary is moved
    let mut engine_process = Command::new(ENGINE_COMMAND)
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

    // Create a channel for the engine listener to send messages to Bevy via events
    let (board_tx, board_rx) = crossbeam_channel::unbounded();

    // Create a thread for parsing the UciMessages and sending them to the engine
    let shared_stdin_clone = shared_stdin.clone();
    std::thread::spawn(move || {
        for message in uci_rx {
            match_uci_message(message, &shared_stdin_clone, &mut reader, &board_tx, &mut engine_process)
                .unwrap_or_else(|e| panic!("{e}"));
        }

        println!("Mpsc Channel Closed");
    });

    UciToBoardReceiver(board_rx)
}

/// # Panics
/// Panics if the best move reply can't be parsed
/// # Errors
/// Returns an error if the stdin cant be locked, flushed, or wrote to
/// Returns an error if the stdout reader cannot read a line
/// Returns an error if mpsc channel cannot be closed
/// Returns an error if the engine process cannot be waited on
pub fn match_uci_message(
    message: UciMessage,
    shared_stdin: &Arc<Mutex<ChildStdin>>,
    stdout_reader: &mut BufReader<ChildStdout>,
    board_tx: &crossbeam_channel::Sender<UciToBoardMessage>,
    engine_process: &mut Child,
) -> Result<(), UciError> {
    match message {
        UciMessage::NewMove {
            move_history,
            player_to_move,
        } => {
            lock_std_and_write(
                shared_stdin,
                // format!("position startpos moves {move_history}"),
                format!("position fen {DEFAULT_FEN} moves {move_history}"),
            )?;

            // Read and print engine output until it reports "readyok"
            uci_is_ready_and_wait(shared_stdin, stdout_reader)?;

            // Tell the engine to find the best move
            let lines = uci_send_message_and_wait_for(shared_stdin, stdout_reader, "go depth 20", |line| {
                line.split_whitespace().next() == Some("bestmove")
            })?;

            // Convert best move string into the equivalent PieceMove
            let move_part = lines[0]
                .trim()
                .trim_start_matches("bestmove")
                .split_whitespace()
                .next()
                .expect("Could not parse algebraic piece move from bestmove reply");
            let piece_move = PieceMove::from_algebraic(move_part).map_err(UciError::PieceMoveParseError)?;

            // Send this move to the board
            board_tx.send(UciToBoardMessage::BestMove(piece_move))?;

            send_uci_info(lines[1].as_str(), board_tx, player_to_move)?;
        }
        UciMessage::UpdateEval {
            move_history,
            player_to_move,
        } => {
            lock_std_and_write(shared_stdin, format!("position fen {DEFAULT_FEN} moves {move_history}"))?;

            // Read and print engine output until it reports "readyok"
            uci_is_ready_and_wait(shared_stdin, stdout_reader)?;

            // Tell the engine to find the best move (but we only care about the information given before the best move)
            let lines = uci_send_message_and_wait_for(shared_stdin, stdout_reader, "go depth 10", |line| {
                line.split_whitespace().next() == Some("bestmove")
            })?;

            send_uci_info(lines[1].as_str(), board_tx, player_to_move)?;
        }
        UciMessage::CloseChannel => {
            // Close the channel
            close_uci_channel()?;

            // Tell the engine to soft exit
            lock_std_and_write(shared_stdin, "quit")?;

            // Wait for the engine process to close
            engine_process.wait().map_err(|_| UciError::EngineProcessWaitError)?;
        }
    }

    Ok(())
}

/// # Errors
/// Returns an error if the Stdin or Stdout cannot be flushed
pub fn greet_uci(stdin: &Arc<Mutex<ChildStdin>>, stdout_reader: &mut BufReader<ChildStdout>) -> Result<(), UciError> {
    // Initialise the engine and await its response of "uciok"
    uci_send_message_and_wait_for(stdin, stdout_reader, "uci", |line| line == "uciok")?;

    // Read and print engine output until it reports "readyok"
    uci_is_ready_and_wait(stdin, stdout_reader)?;

    // Set options for the engine
    lock_std_and_write(stdin, "setoption name Hash value 512")?;
    uci_is_ready_and_wait(stdin, stdout_reader)?;

    // // Set options for the engine
    // lock_std_and_write(stdin, "setoption name Threads value 8")?;
    // uci_is_ready_and_wait(stdin, stdout_reader)?;

    // Read and print engine output until it reports "readyok"
    uci_is_ready_and_wait(stdin, stdout_reader)?;

    Ok(())
}

/// # Errors
/// Returns an error if the ``OnceLock`` cannot be got
/// Returns an error if the ``Mutex`` cannot be locked
/// Returns an error if the message cannot be sent
pub fn transmit_to_uci(message: UciMessage) -> Result<(), UciError> {
    // Get the Mutex from the OnceLock, Lock the Mutex, Then send message via the TX channel (Converting all errors to UciError along the way)
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
pub fn close_uci_channel() -> Result<(), UciError> {
    // Get the Mutex from the OnceLock, Lock the Mutex, Then remove the inner part of the Mutex (Converting all errors to UciError along the way)
    UCI_TX
        .get()
        .ok_or(UciError::OnceLockError)?
        .lock()
        .map_err(|_| UciError::MutexLockError)?
        .take();

    Ok(())
}

/// # Errors
/// Returns an error if the stdin can't be locked, wrote to, and flushed
/// Returns an error if the stdout reader can't read a line
pub fn uci_send_message_and_wait_for(
    stdin: &Arc<Mutex<ChildStdin>>,
    stdout_reader: &mut BufReader<ChildStdout>,
    message: &str,
    wait_function: impl Fn(&str) -> bool,
) -> Result<Vec<String>, UciError> {
    // Write a message to stdin
    lock_std_and_write(stdin, message)?;

    // Read and print engine output until the wait_function is true
    let mut prev_line;
    let mut line = String::new();
    loop {
        // Remember the previous line
        prev_line = line.clone();

        line.clear();
        stdout_reader.read_line(&mut line)?;

        if !line.is_empty() {
            print!("Engine: {line}");

            if wait_function(line.trim()) {
                break;
            }
        }
    }

    Ok(vec![line, prev_line])
}

/// # Errors
/// Returns an error if the ``Stdin`` ``Mutex`` cannot be locked
/// Returns an error if ``Stdin`` cannot be written to or flushed
/// Returns an error if a line from ``Stdout`` ``BufReader`` cannot be read
pub fn uci_is_ready_and_wait(stdin: &Arc<Mutex<ChildStdin>>, stdout_reader: &mut BufReader<ChildStdout>) -> Result<(), UciError> {
    uci_send_message_and_wait_for(stdin, stdout_reader, "isready", |line| line == "readyok").map(|_| ())
}

/// # Errors
/// Returns an error if the ``Stdin`` ``Mutex`` can't be locked
/// Returns an error if the ``Stdin`` can't be written to
/// Returns an error if the ``Stdin`` can't be flushed
pub fn lock_std_and_write<S: std::fmt::Display>(stdin: &Arc<Mutex<ChildStdin>>, message: S) -> Result<(), UciError> {
    {
        let mut locked_stdin = stdin.lock().map_err(|_| UciError::MutexLockError)?;
        writeln!(locked_stdin, "{message}")?;
        locked_stdin.flush()?;
    }

    Ok(())
}
