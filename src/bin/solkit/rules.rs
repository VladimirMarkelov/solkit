use std::collections::HashMap;
use std::path::Path;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use crate::loader;
use solkit::card::{Face, Suit};
use solkit::err::SolError;
use solkit::gconf::{ColConf, Conf, FaceOrder, FndSlot, PileConf, Playable, SuitOrder, TempConf};

// return either pre-defined list of solitaires or a single one loaded from a file
pub(crate) fn load_rules(filename: Option<String>) -> Result<HashMap<String, Conf>, SolError> {
    match filename {
        Some(s) => custom_rule(&s),
        None => builtin_rules(),
    }
}

fn custom_rule(filename: &str) -> Result<HashMap<String, Conf>, SolError> {
    let path = Path::new(filename);
    if !path.is_file() {
        return Err(SolError::InvalidFileName);
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_e) => return Err(SolError::FailedToOpenRules),
    };

    const BOM: [u8; 3] = [0xef, 0xbb, 0xbf];
    let bom = if let Ok(s) = String::from_utf8(BOM.to_vec()) { s } else { "".to_string() };

    let buf = BufReader::new(file);
    let mut lines: Vec<String> = Vec::new();
    for line in buf.lines() {
        let line = match line {
            Err(_e) => return Err(SolError::FailedToOpenRules),
            Ok(l) => l,
        };
        let line = line.trim_start_matches(&bom);
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let low = line.to_lowercase();
        lines.push(low);
    }

    let conf = loader::load_config(&lines)?;
    let mut rules: HashMap<String, Conf> = HashMap::new();
    let name = conf.name.clone();
    rules.insert(name, conf);
    Ok(rules)
}

fn builtin_rules() -> Result<HashMap<String, Conf>, SolError> {
    let mut rules: HashMap<String, Conf> = HashMap::new();

    let conf = Conf {
        name: "Klondike (hard)".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Any,
        pile: Some(PileConf { deal_by: 3, redeals: -1, pile_to_cols: false }),
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 2, up: 1, take_only: false },
            ColConf { count: 3, up: 2, take_only: false },
            ColConf { count: 4, up: 2, take_only: false },
            ColConf { count: 5, up: 3, take_only: false },
            ColConf { count: 6, up: 3, take_only: false },
            ColConf { count: 7, up: 4, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::AlternateColor,
        col_refill: Face::K,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Klondike (easy)".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Any,
        pile: Some(PileConf { deal_by: 1, redeals: -1, pile_to_cols: false }),
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 2, up: 1, take_only: false },
            ColConf { count: 3, up: 2, take_only: false },
            ColConf { count: 4, up: 2, take_only: false },
            ColConf { count: 5, up: 3, take_only: false },
            ColConf { count: 6, up: 3, take_only: false },
            ColConf { count: 7, up: 4, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::AlternateColor,
        col_refill: Face::K,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Klondike (double)".to_string(),
        chance: None,
        deck_count: 2,
        playable: Playable::Any,
        pile: Some(PileConf { deal_by: 3, redeals: -1, pile_to_cols: false }),
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 2, up: 1, take_only: false },
            ColConf { count: 3, up: 2, take_only: false },
            ColConf { count: 4, up: 2, take_only: false },
            ColConf { count: 5, up: 3, take_only: false },
            ColConf { count: 6, up: 3, take_only: false },
            ColConf { count: 7, up: 4, take_only: false },
            ColConf { count: 8, up: 4, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::AlternateColor,
        col_refill: Face::K,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Free cell".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Top,
        pile: None,
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: Some(TempConf { count: 4 }),
        cols: vec![
            ColConf { count: 7, up: 7, take_only: false },
            ColConf { count: 7, up: 7, take_only: false },
            ColConf { count: 7, up: 7, take_only: false },
            ColConf { count: 7, up: 7, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::AlternateColor,
        col_refill: Face::K,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Russian solitaire".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Any,
        pile: None,
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 6, up: 5, take_only: false },
            ColConf { count: 7, up: 5, take_only: false },
            ColConf { count: 8, up: 5, take_only: false },
            ColConf { count: 9, up: 5, take_only: false },
            ColConf { count: 10, up: 5, take_only: false },
            ColConf { count: 11, up: 5, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::AlternateColor,
        col_refill: Face::K,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Pile\'em up".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Top,
        pile: Some(PileConf { deal_by: 0, redeals: 0, pile_to_cols: true }),
        fnd: vec![FndSlot { first: Face::Any, suit: Suit::Any, forder: FaceOrder::Any, sorder: SuitOrder::Any }],
        temp: Some(TempConf { count: 2 }),
        cols: vec![
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
        ],
        col_forder: FaceOrder::Any,
        col_sorder: SuitOrder::Forbid,
        col_refill: Face::Unavail,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "American toad".to_string(),
        chance: None,
        deck_count: 2,
        playable: Playable::Any,
        pile: Some(PileConf { deal_by: 1, redeals: 0, pile_to_cols: false }),
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 20, up: 20, take_only: true },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::SameSuit,
        col_refill: Face::Any,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Auld lang syne".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Top,
        pile: Some(PileConf { deal_by: 1, redeals: 0, pile_to_cols: true }),
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::Forbid,
        col_refill: Face::Any,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Aunt Mary".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Any,
        pile: Some(PileConf { deal_by: 1, redeals: 0, pile_to_cols: false }),
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 5, take_only: false },
            ColConf { count: 6, up: 4, take_only: false },
            ColConf { count: 6, up: 3, take_only: false },
            ColConf { count: 6, up: 2, take_only: false },
            ColConf { count: 6, up: 1, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::AlternateColor,
        col_refill: Face::K,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Batsford".to_string(),
        chance: None,
        deck_count: 2,
        playable: Playable::Any,
        pile: Some(PileConf { deal_by: 1, redeals: 0, pile_to_cols: false }),
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::Any },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 2, up: 1, take_only: false },
            ColConf { count: 3, up: 1, take_only: false },
            ColConf { count: 4, up: 1, take_only: false },
            ColConf { count: 5, up: 1, take_only: false },
            ColConf { count: 6, up: 1, take_only: false },
            ColConf { count: 7, up: 1, take_only: false },
            ColConf { count: 8, up: 1, take_only: false },
            ColConf { count: 9, up: 1, take_only: false },
            ColConf { count: 10, up: 1, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::AlternateColor,
        col_refill: Face::K,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Blind alleys".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Any,
        pile: Some(PileConf { deal_by: 1, redeals: 1, pile_to_cols: false }),
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 3, up: 1, take_only: false },
            ColConf { count: 3, up: 1, take_only: false },
            ColConf { count: 3, up: 1, take_only: false },
            ColConf { count: 3, up: 1, take_only: false },
            ColConf { count: 3, up: 1, take_only: false },
            ColConf { count: 3, up: 1, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::AlternateColor,
        col_refill: Face::Any,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Brigade".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Top,
        pile: None,
        fnd: vec![
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::A, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 6, up: 6, take_only: false },
            ColConf { count: 10, up: 10, take_only: true },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::Any,
        col_refill: Face::Any,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Deuces".to_string(),
        chance: None,
        deck_count: 2,
        playable: Playable::Any,
        pile: Some(PileConf { deal_by: 1, redeals: 0, pile_to_cols: false }),
        fnd: vec![
            FndSlot { first: Face::N2, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::N2, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::N2, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::N2, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::N2, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::N2, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::N2, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::N2, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::SameSuit,
        col_refill: Face::Any,
    };
    rules.insert(conf.name.clone(), conf);

    let conf = Conf {
        name: "Canfield".to_string(),
        chance: None,
        deck_count: 1,
        playable: Playable::Any,
        pile: Some(PileConf { deal_by: 3, redeals: -1, pile_to_cols: false }),
        fnd: vec![
            FndSlot { first: Face::Column, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::Column, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::Column, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
            FndSlot { first: Face::Column, suit: Suit::Any, forder: FaceOrder::Asc, sorder: SuitOrder::SameSuit },
        ],
        temp: None,
        cols: vec![
            ColConf { count: 14, up: 14, take_only: true },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
            ColConf { count: 1, up: 1, take_only: false },
        ],
        col_forder: FaceOrder::Desc,
        col_sorder: SuitOrder::AlternateColor,
        col_refill: Face::Any,
    };
    rules.insert(conf.name.clone(), conf);

    Ok(rules)
}
