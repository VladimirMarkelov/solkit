use crate::card::{Face, Suit};
use crate::err::SolError;

#[derive(Clone, Copy, PartialEq)]
pub enum FaceOrder {
    Asc,
    Desc,
    Any,
}

pub fn str_to_face_order(s: &str) -> Result<FaceOrder, SolError> {
    match s {
        "asc" | "ascending" | "inc" | "increasing" => Ok(FaceOrder::Asc),
        "desc" | "descending" | "dec" | "decreasing" => Ok(FaceOrder::Desc),
        "any" | "alternate" => Ok(FaceOrder::Any),
        _ => Err(SolError::InvalidFaceOrder(s.to_string())),
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SuitOrder {
    SameSuit,
    SameColor,
    AlternateColor,
    Any,
    Forbid,
}

pub fn str_to_suit_order(s: &str) -> Result<SuitOrder, SolError> {
    match s {
        "same" | "same suit" | "samesuit" => Ok(SuitOrder::SameSuit),
        "same color" | "samecolor" => Ok(SuitOrder::SameColor),
        "alternate" | "alternate color" | "alternatecolor" => Ok(SuitOrder::AlternateColor),
        "none" | "forbid" | "disable" => Ok(SuitOrder::Forbid),
        "any" => Ok(SuitOrder::Any),
        _ => Err(SolError::InvalidSuitOrder(s.to_string())),
    }
}

// deck pile configuration
#[derive(Clone, Copy)]
pub struct PileConf {
    pub deal_by: u8,        // how many cards to move from deck to waste at a time
    pub redeals: i8,        // redeals left
    pub pile_to_cols: bool, // deal to columns instead of waste
}

impl PileConf {
    pub fn validate(&self) -> Result<(), SolError> {
        if self.deal_by == 0 || self.deal_by > 16 {
            return Err(SolError::InvalidDealBy(self.deal_by));
        }
        Ok(())
    }
}

// foundation pile configuration
#[derive(Clone, Copy)]
pub struct FndSlot {
    pub first: Face,       // face of the card that starts the pile
    pub suit: Suit,        // suit of the card that starts the pile
    pub forder: FaceOrder, // face order
    pub sorder: SuitOrder, // suit order
}

// free-cell configuration
#[derive(Clone, Copy)]
pub struct TempConf {
    pub count: u8, // the number of free cells
}

impl TempConf {
    pub fn validate(&self) -> Result<(), SolError> {
        if self.count > 4 {
            return Err(SolError::InvalidTempNumber(self.count));
        }
        Ok(())
    }
}

// column configuration
#[derive(Clone, Copy)]
pub struct ColConf {
    pub count: u8,       // initial number of cards
    pub up: u8,          // initial number of face-up cards
    pub take_only: bool, // a user cannot put cards to the pile, only take from it
}

// which cards can be move from a column to another pile
#[derive(Clone, Copy, PartialEq)]
pub enum Playable {
    Top,     // only the top one
    Any,     // any number of face-up cards
    Ordered, // any number of face-up cards from the top of a pile if the cards are in order
}

#[derive(Clone)]
pub struct Conf {
    pub chance: Option<u16>, // chance of winning 1 of N (if known)
    pub name: String,        // solitaire unique name

    pub deck_count: u8,     // number of decks (1 or 2)
    pub playable: Playable, // what cards in a column are playable

    pub pile: Option<PileConf>, // deck and waste configuration
    pub fnd: Vec<FndSlot>,      // foundation configuration
    pub temp: Option<TempConf>, // number of temp slots
    pub cols: Vec<ColConf>,     // column configuration
    pub col_forder: FaceOrder,  // face order of cards in columns
    pub col_sorder: SuitOrder,  // suit order of cards in columns
    // Face of card that must start a pile when it gets empty.
    // Unavail: empty col cannot be filled
    pub col_refill: Face,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            name: String::new(),
            chance: None,
            deck_count: 1,
            playable: Playable::Top,
            pile: None,
            fnd: Vec::new(),
            temp: None,
            cols: Vec::new(),
            col_forder: FaceOrder::Desc,
            col_sorder: SuitOrder::SameSuit,
            col_refill: Face::Unavail,
        }
    }
}

impl Conf {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn validate(&self) -> Result<(), SolError> {
        if self.deck_count == 0 || self.deck_count > 2 {
            return Err(SolError::InvalidDeckNumber(self.deck_count));
        }
        if let Some(ref cfg) = self.temp {
            cfg.validate()?;
        }
        if let Some(ref dc) = self.pile {
            dc.validate()?;
        }
        if self.fnd.is_empty() {
            return Err(SolError::NoFoundation);
        }
        for w in &self.fnd {
            if w.first == Face::Empty {
                return Err(SolError::NoFoundationStart);
            }
        }
        if self.cols.is_empty() {
            return Err(SolError::NoCols);
        }
        if self.cols.len() > 10 {
            return Err(SolError::InvalidColNumber(self.cols.len() as u8));
        }
        Ok(())
    }

    pub fn deal_by(&self) -> u8 {
        if let Some(ref pconf) = self.pile {
            pconf.deal_by
        } else {
            0
        }
    }

    pub fn redeals(&self) -> i8 {
        if let Some(ref pconf) = self.pile {
            pconf.redeals
        } else {
            0
        }
    }
}
