use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolError {
    #[error("Invalid number of decks: {0}. Must be 1 or 2")]
    InvalidDeckNumber(u8),
    #[error("Invalid number of temporary slots: {0}. Must be between 0 and 4")]
    InvalidTempNumber(u8),
    #[error("Invalid number of columns: {0}. Must be between 1 and 10")]
    InvalidColNumber(u8),
    #[error("Invalid number of dealt cards at a time: {0}. Must be between 1 and 16")]
    InvalidDealBy(u8),
    #[error("Invalid number of redeals: {0}")]
    InvalidRedeals(i8),
    #[error("No foundation defined")]
    NoFoundation,
    #[error("Foundation cannot start with EMPTY card face")]
    NoFoundationStart,
    #[error("No columns defined")]
    NoCols,
    #[error("Insufficient cards in the deck for {0}")]
    InsufficientFor(String),
    #[error("Pile is disabled but a few cards are still in the deck")]
    UnusedCards,
    #[error("Invalid location")]
    InvalidLocation,
    #[error("Invalid destination")]
    InvalidDestination,
    #[error("Invalid move")]
    InvalidMove,
    #[error("Card cannot be moved")]
    Unplayable,
    #[error("No card selected")]
    NotSelected,
    #[error("Card cannot be played")]
    NoDestination,
    #[error("Invalid card suit: {0}")]
    InvalidSuit(String),
    #[error("Invalid card face: {0}")]
    InvalidFace(String),
    #[error("Invalid card suit order: {0}")]
    InvalidSuitOrder(String),
    #[error("Invalid card face order: {0}")]
    InvalidFaceOrder(String),
    #[error("Solitaire list is empty")]
    SolitaireListEmpty,

    #[error("Invalid configuration line: {0}")]
    InvalidConfLine(String),
    #[error("Invalid configuration section {0}")]
    InvalidConfSection(String),
    #[error("Invalid configuration option {1} of section {0}")]
    InvalidConfOption(String, String),
    #[error("Invalid configuration value {1} for option {0}")]
    InvalidConfOptionValue(String, String),
    #[error("Invalid configuration: limit for a temp slot must be set if temp is refillable")]
    InvalidConfTempLimit,
    #[error("Invalid configuration: refillable temp slot must define sort order for card face and/or for card suit")]
    InvalidConfTempOrder,
    #[error("Invalid temp configuration: only one slot can be refillable")]
    InvalidConfTempSingleRefillable,
    #[error("File does not exist")]
    InvalidFileName,
    #[error("Reading rules from file failed")]
    FailedToOpenRules,

    #[error("{0}")]
    Unexpected(String), // for third-party errors
    #[error("{0} unsupported yet")]
    Unsupported(String),
}
