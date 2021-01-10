use solkit::card::{str_to_face, str_to_suit, Card, Face, Suit};
use solkit::err::SolError;
use solkit::gconf::{
    str_to_face_order, str_to_suit_order, ColConf, Conf, FaceOrder, FndSlot, PileConf, Playable, SuitOrder, TempConf,
};

// load a solitaire rules from a UTF-8 text file
pub(crate) fn load_config(data: &[String]) -> Result<Conf, SolError> {
    let mut conf = Conf::new();
    let mut idx = 0usize;
    while idx < data.len() {
        let low = &data[idx];
        if !low.starts_with('[') {
            return Err(SolError::InvalidConfLine(low.to_string()));
        }
        let section_name = low.trim_matches(|c| c == '[' || c == ']' || c == ' ');
        match section_name {
            "global" => idx = parse_global(&mut conf, data, idx + 1)?,
            "deck" => idx = parse_deck(&mut conf, data, idx + 1)?,
            "foundation" => idx = parse_foundation(&mut conf, data, idx + 1)?,
            "temp" => idx = parse_temp(&mut conf, data, idx + 1)?,
            "column" => idx = parse_play(&mut conf, data, idx + 1)?,
            _ => return Err(SolError::InvalidConfSection(section_name.to_string())),
        }
    }
    Ok(conf)
}

fn parse_global(conf: &mut Conf, data: &[String], idx: usize) -> Result<usize, SolError> {
    let mut idx = idx;
    while idx < data.len() {
        let low = &data[idx];
        idx += 1;
        if low.starts_with('[') {
            return Ok(idx - 1);
        }
        let pos = match low.find('=') {
            Some(n) => n,
            None => return Err(SolError::InvalidConfLine(low.to_string())),
        };
        let opt_name = low[..pos].trim();
        let opt_val = low[pos + 1..].trim();
        match opt_name {
            "name" => conf.name = opt_val.to_string(),
            "chance" => match opt_val.parse::<u16>() {
                Ok(n) => conf.chance = Some(n),
                Err(_) => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
            },
            "decks" => match opt_val.parse::<u8>() {
                Ok(n) if n == 1 || n == 2 => conf.deck_count = n,
                _ => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
            },
            _ => return Err(SolError::InvalidConfOption("global".to_string(), opt_name.to_string())),
        }
    }
    Ok(idx)
}

fn parse_deck(conf: &mut Conf, data: &[String], idx: usize) -> Result<usize, SolError> {
    let mut idx = idx;
    let mut pconf = PileConf { deal_by: 0, redeals: 0, pile_to_cols: false };
    while idx < data.len() {
        let low = &data[idx];
        idx += 1;
        if low.starts_with('[') {
            pconf.validate()?;
            conf.pile = Some(pconf);
            return Ok(idx - 1);
        }
        let pos = match low.find('=') {
            Some(n) => n,
            None => return Err(SolError::InvalidConfLine(low.to_string())),
        };
        let opt_name = low[..pos].trim();
        let opt_val = low[pos + 1..].trim();
        match opt_name {
            "redeals" => {
                if opt_val == "unlimited" {
                    pconf.redeals = -1;
                } else {
                    match opt_val.parse::<i8>() {
                        Ok(n) => pconf.redeals = n,
                        _ => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
                    }
                }
            }
            "deal_by" => match opt_val.parse::<u8>() {
                Ok(n) => pconf.deal_by = n,
                _ => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
            },
            "deal_to" => match opt_val {
                "deck" | "side" | "waste" => pconf.pile_to_cols = false,
                "column" | "columns" => pconf.pile_to_cols = true,
                _ => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
            },
            _ => return Err(SolError::InvalidConfOption("deck".to_string(), opt_name.to_string())),
        }
    }
    pconf.validate()?;
    conf.pile = Some(pconf);
    Ok(idx)
}

fn parse_foundation(conf: &mut Conf, data: &[String], idx: usize) -> Result<usize, SolError> {
    let mut idx = idx;
    let mut fnd: Vec<FndSlot> = Vec::new();
    while idx < data.len() {
        let low = &data[idx];
        idx += 1;
        if low.starts_with('[') {
            if fnd.is_empty() {
                return Err(SolError::NoFoundation);
            }
            conf.fnd = fnd;
            return Ok(idx - 1);
        }
        let pos = match low.find('=') {
            Some(n) => n,
            None => return Err(SolError::InvalidConfLine(low.to_string())),
        };
        let opt_name = low[..pos].trim();
        let opt_val = low[pos + 1..].trim();
        match opt_name {
            "column" => {
                let slot = parse_fnd_slot(&opt_val)?;
                fnd.push(slot);
            }
            _ => return Err(SolError::InvalidConfOption("foundation".to_string(), opt_name.to_string())),
        }
    }
    if fnd.is_empty() {
        return Err(SolError::NoFoundation);
    }
    conf.fnd = fnd;
    Ok(idx)
}

fn parse_fnd_slot(s: &str) -> Result<FndSlot, SolError> {
    let mut slot = FndSlot {
        first: Face::Any,
        suit: Suit::Any,
        forder: FaceOrder::Asc,
        sorder: SuitOrder::SameSuit,
        filler: None,
    };
    let v: Vec<&str> = s.split(',').collect();
    if v.len() != 4 && v.len() != 6 {
        return Err(SolError::InvalidConfOptionValue("column".to_string(), s.to_string()));
    }
    slot.first = str_to_face(v[0].trim())?;
    slot.suit = str_to_suit(v[1].trim())?;
    slot.forder = str_to_face_order(v[2].trim())?;
    slot.sorder = str_to_suit_order(v[3].trim())?;
    if v.len() == 6 {
        let face = str_to_face(v[4].trim())?;
        let suit = str_to_suit(v[4].trim())?;
        let card = Card::new(suit, face);
        if !card.is_regular() {
            return Err(SolError::InvalidConfOptionValue("column initial card".to_string(), s.to_string()));
        }
        slot.filler = Some(card);
    }
    Ok(slot)
}

fn parse_temp(conf: &mut Conf, data: &[String], idx: usize) -> Result<usize, SolError> {
    let mut idx = idx;
    let mut tconf = TempConf { count: 0 };
    while idx < data.len() {
        let low = &data[idx];
        idx += 1;
        if low.starts_with('[') {
            if tconf.count != 0 {
                tconf.validate()?;
                conf.temp = Some(tconf);
            }
            return Ok(idx - 1);
        }
        let pos = match low.find('=') {
            Some(n) => n,
            None => return Err(SolError::InvalidConfLine(low.to_string())),
        };
        let opt_name = low[..pos].trim();
        let opt_val = low[pos + 1..].trim();
        match opt_name {
            "slots" => match opt_val.parse::<u8>() {
                Ok(n) => tconf.count = n,
                _ => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
            },
            _ => return Err(SolError::InvalidConfOption("temp".to_string(), opt_name.to_string())),
        }
    }
    if tconf.count != 0 {
        tconf.validate()?;
        conf.temp = Some(tconf);
    }
    Ok(idx)
}

fn parse_play(conf: &mut Conf, data: &[String], idx: usize) -> Result<usize, SolError> {
    let mut idx = idx;
    while idx < data.len() {
        let low = &data[idx];
        idx += 1;
        if low.starts_with('[') {
            return Ok(idx - 1);
        }
        let pos = match low.find('=') {
            Some(n) => n,
            None => return Err(SolError::InvalidConfLine(low.to_string())),
        };
        let opt_name = low[..pos].trim();
        let opt_val = low[pos + 1..].trim();
        match opt_name {
            "playable_card" => match opt_val {
                "top" => conf.playable = Playable::Top,
                "any" => conf.playable = Playable::Any,
                "order" | "ordered" => conf.playable = Playable::Ordered,
                _ => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
            },
            "refill" => conf.col_refill = str_to_face(opt_val)?,
            "order" => {
                let v: Vec<&str> = opt_val.split(',').collect();
                if v.len() != 2 {
                    return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string()));
                }
                conf.col_forder = str_to_face_order(v[0].trim())?;
                conf.col_sorder = str_to_suit_order(v[1].trim())?;
            }
            "column" => {
                let v: Vec<&str> = opt_val.split(',').collect();
                if v.len() != 2 && v.len() != 3 {
                    return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string()));
                }
                let count = match v[0].trim().parse::<u8>() {
                    Ok(n) => n,
                    Err(_) => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
                };
                let up = match v[1].trim().parse::<u8>() {
                    Ok(n) => n,
                    Err(_) => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
                };
                let take_only = if v.len() != 3 {
                    false
                } else {
                    match v[2].trim() {
                        "take" | "takeonly" | "take only" => true,
                        _ => return Err(SolError::InvalidConfOptionValue(opt_name.to_string(), opt_val.to_string())),
                    }
                };
                conf.cols.push(ColConf { count, up, take_only });
            }
            _ => return Err(SolError::InvalidConfOption("foundation".to_string(), opt_name.to_string())),
        }
    }
    if conf.cols.is_empty() {
        return Err(SolError::NoCols);
    }
    Ok(idx)
}
