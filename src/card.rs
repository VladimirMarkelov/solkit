use nanorand::{WyRand, RNG};

use crate::err::SolError;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Suit {
    Spade,
    Club,
    Diamond,
    Heart,
    Any,
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Face {
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N10,
    J,
    Q,
    K,
    A,
    Empty,
    Any,
    Unavail,
    Column,
}

fn i8_to_suit(s: i8) -> Suit {
    match s {
        0 => Suit::Spade,
        1 => Suit::Club,
        2 => Suit::Diamond,
        3 => Suit::Heart,
        _ => panic!("Invalid suit ID"),
    }
}
pub fn str_to_suit(s: &str) -> Result<Suit, SolError> {
    match s {
        "spade" | "spades" => Ok(Suit::Spade),
        "club" | "clubs" => Ok(Suit::Club),
        "diamond" | "diamonds" => Ok(Suit::Diamond),
        "heart" | "hearts" => Ok(Suit::Heart),
        "any" => Ok(Suit::Any),
        _ => Err(SolError::InvalidSuit(s.to_string())),
    }
}
fn suit_to_i8(s: Suit) -> i8 {
    match s {
        Suit::Spade => 0,
        Suit::Club => 1,
        Suit::Diamond => 2,
        Suit::Heart => 3,
        Suit::Any => panic!("Invalid suit ID"),
    }
}
fn i8_to_face(f: i8) -> Face {
    match f {
        0 => Face::A,
        1 => Face::N2,
        2 => Face::N3,
        3 => Face::N4,
        4 => Face::N5,
        5 => Face::N6,
        6 => Face::N7,
        7 => Face::N8,
        8 => Face::N9,
        9 => Face::N10,
        10 => Face::J,
        11 => Face::Q,
        12 => Face::K,
        _ => panic!("Invalid face ID"),
    }
}
pub fn str_to_face(f: &str) -> Result<Face, SolError> {
    match f {
        "2" => Ok(Face::N2),
        "3" => Ok(Face::N3),
        "4" => Ok(Face::N4),
        "5" => Ok(Face::N5),
        "6" => Ok(Face::N6),
        "7" => Ok(Face::N7),
        "8" => Ok(Face::N8),
        "9" => Ok(Face::N9),
        "10" => Ok(Face::N10),
        "j" => Ok(Face::J),
        "q" => Ok(Face::Q),
        "k" => Ok(Face::K),
        "a" => Ok(Face::A),
        "any" => Ok(Face::Any),
        "empty" => Ok(Face::Empty),
        "first" | "column" | "random" => Ok(Face::Column),
        "unavail" | "unavailable" | "none" => Ok(Face::Unavail),
        _ => Err(SolError::InvalidFace(f.to_string())),
    }
}
fn face_to_i8(f: Face) -> i8 {
    match f {
        Face::A => 0,
        Face::N2 => 1,
        Face::N3 => 2,
        Face::N4 => 3,
        Face::N5 => 4,
        Face::N6 => 5,
        Face::N7 => 6,
        Face::N8 => 7,
        Face::N9 => 8,
        Face::N10 => 9,
        Face::J => 10,
        Face::Q => 11,
        Face::K => 12,
        _ => panic!("Invalid face"),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Card {
    pub face: Face,
    pub suit: Suit,
    pub up: bool,
}

impl Card {
    pub fn new(suit: Suit, face: Face) -> Card {
        Card { up: false, suit, face }
    }
    pub fn new_empty() -> Card {
        Card { up: false, suit: Suit::Any, face: Face::Empty }
    }
    pub fn is_same_suit(&self, other: &Card) -> bool {
        self.suit == other.suit || self.suit == Suit::Any || other.suit == Suit::Any
    }
    pub fn is_same_color(&self, other: &Card) -> bool {
        let this_red = self.suit == Suit::Diamond || self.suit == Suit::Heart;
        let that_red = other.suit == Suit::Diamond || other.suit == Suit::Heart;
        (this_red && that_red) || (!this_red && !that_red)
    }
    // positive difference means other's face value is higher
    pub fn diff(&self, other: &Card) -> i8 {
        if other.face == Face::A && self.face == Face::K {
            return 1;
        }
        if self.face == Face::A && other.face == Face::K {
            return -1;
        }
        face_to_i8(other.face) - face_to_i8(self.face)
    }
    pub fn is_empty(&self) -> bool {
        self.face == Face::Empty
    }
    pub fn is_up(&self) -> bool {
        self.up
    }
}

pub struct Deck {
    cards: Vec<Card>,
    idx: usize,
}

impl Deck {
    pub fn new(count: u8) -> Result<Deck, SolError> {
        if !(1..=2).contains(&count) {
            return Err(SolError::InvalidDeckNumber(count));
        }
        let mut dck: Deck = Deck { cards: Vec::new(), idx: 0 };
        for _d in 0..count {
            for s in suit_to_i8(Suit::Spade)..=suit_to_i8(Suit::Heart) {
                for f in 0..=face_to_i8(Face::K) {
                    dck.cards.push(Card::new(i8_to_suit(s), i8_to_face(f)));
                }
            }
        }
        dck.shuffle();
        Ok(dck)
    }

    fn shuffle(&mut self) {
        let mut rng = WyRand::new();
        rng.shuffle(&mut self.cards);
    }
    pub fn is_empty(&self) -> bool {
        self.idx >= self.cards.len()
    }
    pub fn deal_card(&mut self) -> Option<Card> {
        if self.is_empty() {
            return None;
        }
        self.idx += 1;
        Some(self.cards[self.idx - 1])
    }
}

#[cfg(test)]
mod card_test {
    use super::*;
    #[test]
    fn diff_test() {
        struct Df {
            f: Card,
            s: Card,
            d: i8,
        }
        let whats: Vec<Df> = vec![
            Df { f: Card::new(Suit::Spade, Face::A), s: Card::new(Suit::Spade, Face::N2), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N2), s: Card::new(Suit::Spade, Face::N3), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N3), s: Card::new(Suit::Spade, Face::N4), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N4), s: Card::new(Suit::Spade, Face::N5), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N5), s: Card::new(Suit::Spade, Face::N6), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N6), s: Card::new(Suit::Spade, Face::N7), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N7), s: Card::new(Suit::Spade, Face::N8), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N8), s: Card::new(Suit::Spade, Face::N9), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N9), s: Card::new(Suit::Spade, Face::N10), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N10), s: Card::new(Suit::Spade, Face::J), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::J), s: Card::new(Suit::Spade, Face::Q), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::Q), s: Card::new(Suit::Spade, Face::K), d: 1 },
            Df { f: Card::new(Suit::Spade, Face::N3), s: Card::new(Suit::Spade, Face::N6), d: 3 },
        ];
        for df in whats.iter() {
            let diff = df.f.diff(&df.s);
            assert_eq!(diff, df.d);
            let diff = df.s.diff(&df.f);
            assert_eq!(diff, -df.d);
        }
    }

    #[test]
    fn facei8() {
        for face_id in 0i8..12i8 {
            let face = i8_to_face(face_id);
            let id = face_to_i8(face);
            assert_eq!(face_id, id);
        }
    }

    #[test]
    fn suiti8() {
        for suit_id in 0i8..3i8 {
            let suit = i8_to_suit(suit_id);
            let id = suit_to_i8(suit);
            assert_eq!(suit_id, id);
        }
    }
}
