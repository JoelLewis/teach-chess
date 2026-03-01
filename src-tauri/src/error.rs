use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Game(#[from] GameError),

    #[error(transparent)]
    Engine(#[from] EngineError),

    #[error(transparent)]
    Database(#[from] DatabaseError),

    #[error("Lock poisoned: {0}")]
    Lock(String),
}

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("No active game")]
    NoActiveGame,

    #[error("Game is already over")]
    GameOver,

    #[error("Not your turn")]
    NotYourTurn,

    #[error("Illegal move: {0}")]
    IllegalMove(String),

    #[error("Invalid FEN: {0}")]
    InvalidFen(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Engine not running")]
    NotRunning,

    #[error("Engine already running")]
    AlreadyRunning,

    #[error("Engine process failed: {0}")]
    ProcessError(String),

    #[error("UCI protocol error: {0}")]
    UciError(String),

    #[error("Engine timeout")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Player not found: {0}")]
    PlayerNotFound(String),

    #[error("Game not found: {0}")]
    GameNotFound(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

// Tauri requires errors to be Serialize
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
