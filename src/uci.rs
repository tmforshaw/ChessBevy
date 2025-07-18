use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{mpsc, Arc, Mutex, OnceLock},
};

use chess_core::piece_move::PieceMove;
use thiserror::Error;

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
    MpscSendError(#[from] mpsc::SendError<UciMessage>),

    #[error("Could not find UCI_TX")]
    TxNotFound,

    #[error("Piece move could not be parsed for UCI: {0}")]
    PieceMoveParseError(String),

    #[error("Engine process could not be waited on")]
    EngineProcessWaitError,
}

#[derive(Debug)]
pub enum UciMessage {
    NewMove { move_history: String },
    CloseChannel,
}

/// # Panics
/// Panics if the engine process cannot start
pub fn communicate_to_uci() {
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

    // Create a channel to communicate with this process when it is looping
    let (tx, rx) = std::sync::mpsc::channel();
    UCI_TX
        .set(Mutex::new(Some(tx)))
        .expect("Could not set the UCI_TX OnceLock");

    let shared_stdin_clone = shared_stdin.clone();
    // Create a thread for parsing the messages
    std::thread::spawn(move || {
        for message in rx {
            match_uci_message(
                message,
                &shared_stdin_clone,
                &mut reader,
                &mut engine_process,
            )
            .unwrap_or_else(|e| panic!("{e}"));
        }

        println!("Mpsc Channel Closed");
    });

    transmit_to_uci(UciMessage::NewMove {
        move_history: "e2e4 e7e5".to_string(),
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // transmit_to_uci(UciMessage::CloseChannel).unwrap_or_else(|e| panic!("{e}"));
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
            println!("{piece_move}\t{}", piece_move.to_algebraic().unwrap());

            // Send this move to the board
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
