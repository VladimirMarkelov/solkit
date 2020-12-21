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

#[derive(Clone, Copy)]
pub struct PileConf {
    pub deal_by: u8, //
    pub redeals: i8,
    pub pile_to_cols: bool, // deal to piles instead of pile-up
}

impl PileConf {
    pub fn validate(&self) -> Result<(), SolError> {
        if self.deal_by == 0 || self.deal_by > 16 {
            return Err(SolError::InvalidDealBy(self.deal_by));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct FndSlot {
    pub first: Face,
    pub suit: Suit,
    pub forder: FaceOrder,
    pub sorder: SuitOrder,
}

#[derive(Clone, Copy)]
pub struct TempConf {
    pub count: u8,
}

impl TempConf {
    pub fn validate(&self) -> Result<(), SolError> {
        if self.count > 4 {
            return Err(SolError::InvalidTempNumber(self.count));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct ColConf {
    pub count: u8,
    pub up: u8,
    pub take_only: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Playable {
    Top,
    Any,
    Ordered,
}

#[derive(Clone)]
pub struct Conf {
    pub chance: Option<u16>, // chance of winning 1 of N (if known)
    pub name: String,

    pub deck_count: u8,       // number of decks (1 or 2)
    pub playable: Playable,   // what cards in a column are playable
    pub temp_take_only: bool, // a user can only take cards from temp or put as well

    pub pile: Option<PileConf>,
    pub fnd: Vec<FndSlot>,
    pub temp: Option<TempConf>, // number of temp slots
    pub cols: Vec<ColConf>,
    pub col_forder: FaceOrder,
    pub col_sorder: SuitOrder,
    pub col_refill: Face, // Unavail: empty col cannot be filled
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            name: String::new(),
            chance: None,
            deck_count: 1,
            playable: Playable::Top,
            temp_take_only: false,
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
