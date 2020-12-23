use log::info;
use std::convert::From;

use crate::card::{Card, Deck, Face, Suit};
use crate::err::SolError;
use crate::gconf::{Conf, FaceOrder, Playable, SuitOrder};

pub const ANY_COL: usize = 9999;

type UndoList = Vec<Undo>;
type CardList = Vec<Card>;

#[derive(Clone, Copy)]
pub struct SlotConf {
    // pile can be selected (false only for deck pile)
    pub selectable: bool,
    // all cards in pile are face-up (used by undo).
    // all_up=false & selectable=false means that all cards always are face down
    pub all_up: bool,
    // card can be put in pile if it gets empty (always false for deck pile)
    pub refill: bool,
    // first card face to put into empty pile
    pub start_face: Face,
    // first card suit to put into empty pile
    pub start_suit: Suit,
    // what suits can be put on the top pile card
    pub suit_order: SuitOrder,
    // what faces can be put on the top pile card
    pub face_order: FaceOrder,
    // what cards playable
    pub playable: Playable,
    // flip card automatically if top card is face-down
    pub flip: bool,
    // cannot put card to this pile from any other one
    pub take_only: bool,
    // draw top card only or all in a tall column
    pub draw_all: bool,
}

impl SlotConf {
    // default settings for a foundation pile
    pub fn new_for_fnd() -> Self {
        SlotConf {
            selectable: true,
            all_up: true,
            refill: true,
            start_face: Face::A,
            start_suit: Suit::Any,
            suit_order: SuitOrder::SameSuit,
            face_order: FaceOrder::Asc,
            playable: Playable::Top,
            flip: false,
            take_only: false,
            draw_all: false,
        }
    }
    // default settings for a column pile
    pub fn new_for_col() -> Self {
        SlotConf {
            selectable: true,
            all_up: false,
            refill: true,
            start_face: Face::K,
            start_suit: Suit::Any,
            suit_order: SuitOrder::SameSuit,
            face_order: FaceOrder::Desc,
            playable: Playable::Any,
            flip: true,
            take_only: false,
            draw_all: true,
        }
    }
    // default settings for a free-cell pile
    pub fn new_for_temp() -> Self {
        SlotConf {
            selectable: true,
            all_up: true,
            refill: true,
            start_face: Face::Any,
            start_suit: Suit::Any,
            suit_order: SuitOrder::Any,
            face_order: FaceOrder::Any,
            playable: Playable::Top,
            flip: false,
            take_only: false,
            draw_all: false,
        }
    }
    // default settings for deck and waste piles
    pub fn new_for_pile() -> Self {
        SlotConf {
            selectable: true,
            all_up: false,
            refill: false,
            start_face: Face::Any,
            start_suit: Suit::Any,
            suit_order: SuitOrder::Any,
            face_order: FaceOrder::Any,
            playable: Playable::Top,
            flip: false,
            take_only: true,
            draw_all: false,
        }
    }
}

#[derive(Clone)]
pub struct Pile {
    pub conf: SlotConf,
    pub cards: CardList,
}

// game snapshot
struct Undo {
    redeals: i8,
    piles: Vec<CardList>,
    selected: Pos,
}

// cursor movement direction
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    ColUp(usize),
    ColDown(usize),
    Waste,
    Pile,
    Temp,
}

// cursor position in a play area
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Pos {
    pub col: usize,
    pub row: usize,
}

impl Default for Pos {
    fn default() -> Pos {
        Pos { col: ANY_COL, row: 0 }
    }
}

impl Pos {
    pub fn new() -> Pos {
        Default::default()
    }
    pub fn is_empty(&self) -> bool {
        self.col == ANY_COL
    }
}

pub struct Game<'a> {
    conf: &'a Conf, // selected solitaire rules
    deck: Deck, // a deck
    selected: Pos, // position of the cursor in play area
    undo: UndoList, // list of game snapshots
    piles: Vec<Pile>, // order: fnd, cols, temp, pile
    redeals: i8, // redeals left
}

impl<'a> Game<'a> {
    pub fn init(conf: &'a Conf) -> Result<Game<'a>, SolError> {
        let deck = Deck::new(conf.deck_count)?;
        let redeals = conf.redeals();
        let mut g = Game { conf, deck, undo: Vec::new(), piles: Vec::new(), selected: Pos::new(), redeals };
        g.init_cols()?;
        // at start the current card is always the first one in the first column
        g.selected = Pos { col: g.first_col().unwrap(), row: 0 };
        Ok(g)
    }

    pub fn redeal_left(&self) -> i8 {
        self.redeals
    }

    // can the position be selected with cursor (can the card be played during this move)
    pub fn is_selectable(&self, pos: Option<Pos>) -> bool {
        let pos = if let Some(p) = pos { p } else { self.selected };
        if pos.col >= self.piles.len() {
            false
        } else {
            if self.conf.playable == Playable::Top && pos.row != 0 {
                return false;
            }
            if self.conf.playable == Playable::Ordered && pos.row >= self.ordered_count(pos.col) {
                return false;
            }
            if !self.piles[pos.col].conf.selectable {
                return false;
            }
            let l = self.piles[pos.col].cards.len();
            if l == 0 || pos.row >= l {
                return false;
            }
            self.piles[pos.col].cards[l - 1 - pos.row].is_up()
        }
    }

    // sets the cursor to the card if it is playable
    pub fn select(&mut self, pos: Pos) {
        if !self.is_selectable(Some(pos)) {
            return;
        }
        self.selected = pos;
    }

    // if Space/Enter/Left-Mouse clicked when the selected card is deck
    pub fn is_deck_clicked(&self, pos: Option<Pos>) -> bool {
        let pos = if let Some(p) = pos { p } else { self.selected };
        match self.first_pile() {
            Some(n) => pos.col == n,
            None => false,
        }
    }

    // number of columns in play area
    pub fn column_count(&self) -> usize {
        self.piles.len()
    }

    // the number of top cards of a column that are already in correct order
    pub fn ordered_count(&self, pile_id: usize) -> usize {
        let col_start = self.first_col().unwrap(); // TODO:
        let col_cnt = self.col_count();
        if pile_id < col_start || pile_id >= col_start + col_cnt {
            return 0;
        }
        let pile = &self.piles[pile_id];
        if pile.cards.len() < 2 {
            return pile.cards.len();
        }
        let mut cnt = 1usize;
        let mut idx = pile.cards.len() - 1;
        while idx > 0 {
            let ok_suit = match pile.conf.suit_order {
                SuitOrder::SameSuit => pile.cards[idx].is_same_suit(&pile.cards[idx - 1]),
                SuitOrder::SameColor => pile.cards[idx].is_same_color(&pile.cards[idx - 1]),
                SuitOrder::AlternateColor => !pile.cards[idx].is_same_color(&pile.cards[idx - 1]),
                SuitOrder::Any => true,
                SuitOrder::Forbid => false,
            };
            let diff = pile.cards[idx].diff(&pile.cards[idx - 1]);
            let ok_face = match pile.conf.face_order {
                FaceOrder::Asc => diff == -1,
                FaceOrder::Desc => diff == 1,
                FaceOrder::Any => diff == 1 || diff == -1,
            };
            if ok_suit && ok_face {
                cnt += 1;
                idx -= 1;
            } else {
                break;
            }
        }
        cnt
    }

    // returns true if the card at position pos(or a few cards if the card is not the top one) can
    // be moved to the top of pile with ID pile_id
    fn can_move(&self, pos: Pos, pile_id: usize) -> bool {
        let card = self.card_at(pos);
        if !card.up || card.is_empty() || pile_id >= self.piles.len() {
            return false;
        }
        let pile = &self.piles[pile_id];
        if pile.conf.playable == Playable::Top && pos.row != 0 {
            return false;
        }
        if pile.conf.take_only || (!pile.conf.refill && pile.cards.is_empty()) {
            return false;
        }

        if pile.cards.is_empty() {
            let ok_suit = pile.conf.start_suit == Suit::Any || pile.conf.start_suit == card.suit;
            let ok_face = pile.conf.start_face == Face::Any || pile.conf.start_face == card.face;
            return ok_suit && ok_face;
        }

        let top = &pile.cards[pile.cards.len() - 1];
        let ok_suit = match pile.conf.suit_order {
            SuitOrder::SameSuit => card.is_same_suit(top),
            SuitOrder::SameColor => card.is_same_color(top),
            SuitOrder::AlternateColor => !card.is_same_color(top),
            SuitOrder::Any => true,
            SuitOrder::Forbid => false,
        };
        let diff = card.diff(top);
        let ok_face = match pile.conf.face_order {
            FaceOrder::Asc => diff == -1,
            FaceOrder::Desc => diff == 1,
            FaceOrder::Any => diff == 1 || diff == -1,
        };
        ok_face && ok_suit
    }

    // return the currently selected card
    pub fn selected_loc(&self) -> Pos {
        self.selected
    }

    // return the ID of the deck
    pub fn first_pile(&self) -> Option<usize> {
        self.conf.pile.map(|_| self.fnd_count() + self.col_count() + self.temp_count())
    }

    // return the ID of the first free-cell pile
    pub fn first_temp(&self) -> Option<usize> {
        self.conf.temp.map(|_| self.fnd_count() + self.col_count())
    }

    // return the ID of the first foundation pile
    pub fn first_fnd(&self) -> Option<usize> {
        // Fnd always exists and it is always the first
        Some(0)
    }

    // return the ID of the first columns
    pub fn first_col(&self) -> Option<usize> {
        // Cols always exist and it goes after the fnd
        Some(self.fnd_count())
    }

    // return the ID of the first cards in the pile that is face-up. All cards after it are always
    // face-up ones as well.
    fn first_up(&self, col_idx: usize) -> usize {
        let mut mx = 0usize;
        let l = self.piles[col_idx].cards.len();
        for (idx, c) in self.piles[col_idx].cards.iter().enumerate() {
            mx = l - idx - 1;
            if c.is_up() {
                break;
            }
        }
        mx
    }

    // select the upper card in a pile if the card is playable. Do not move cursor otherwise
    fn do_up_for_col(&self, col_idx: usize, curr: usize) -> usize {
        if self.piles[col_idx].cards.len() < 2 {
            return curr;
        }
        let mx = self.first_up(col_idx);
        if curr >= mx {
            curr
        } else {
            curr + 1
        }
    }

    // select the next pile of the same type. If it is the last one, selects the first pile of the
    // same type(looped selection).
    fn move_next_pile_of_type(&mut self, first: usize, count: usize) {
        if self.selected.col < first || self.selected.col >= first + count {
            self.selected = Pos { col: first, row: 0 };
            return;
        }
        if self.selected.col == first + count - 1 {
            self.selected = Pos { col: first, row: 0 };
        } else {
            self.selected.col += 1;
        }
    }

    // move cursor to the defined direction
    pub fn move_selection(&mut self, dir: Direction) -> bool {
        match dir {
            Direction::Right => {
                let col = self.selected.col;
                if col == self.piles.len() - 1 {
                    self.selected = Pos { col: 0, row: 0 };
                } else {
                    self.selected = Pos { col: col + 1, row: 0 };
                }
            }
            Direction::Left => {
                let col = self.selected.col;
                if col == 0 {
                    self.selected = Pos { col: self.piles.len() - 1, row: 0 };
                } else {
                    self.selected = Pos { col: col - 1, row: 0 };
                }
            }
            Direction::Down => {
                let row = self.selected.row;
                if row == 0 {
                    return false;
                }
                let col_first = self.first_col().unwrap();
                let col_total = self.col_count();
                if self.piles[self.selected.col].conf.take_only {
                    return false;
                }
                if self.selected.col < col_first || self.selected.col >= col_first + col_total {
                    return false;
                }
                self.selected.row = row - 1;
            }
            Direction::Up => {
                let row = self.selected.row;
                let col_first = self.first_col().unwrap();
                let col_total = self.col_count();
                if self.piles[self.selected.col].conf.take_only {
                    return false;
                }
                if self.selected.col < col_first || self.selected.col >= col_first + col_total {
                    return false;
                }
                let new_row = self.do_up_for_col(self.selected.col, row);
                let new_pos = Some(Pos { col: self.selected.col, row: new_row });
                if new_row > self.selected.row && !self.is_selectable(new_pos) {
                    return false;
                }
                if row == new_row {
                    return false;
                }
                self.selected.row = new_row;
            }
            Direction::ColUp(n) => {
                let col_first = self.first_col().unwrap();
                let col_total = self.col_count();
                if n >= col_total {
                    return false;
                }
                let new_col = col_first + n;
                if self.selected.col != new_col || self.piles[new_col].cards.len() < 2 {
                    self.selected = Pos { col: new_col, row: 0 };
                    return true;
                }
                if self.piles[self.selected.col].conf.take_only {
                    return false;
                }
                let new_row = self.do_up_for_col(new_col, self.selected.row);
                let new_pos = Some(Pos { col: self.selected.col, row: new_row });
                if new_row > self.selected.row && !self.is_selectable(new_pos) {
                    return false;
                }
                if self.selected.row != new_row {
                    self.selected.row = new_row;
                } else {
                    self.selected.row = 0;
                }
            }
            Direction::ColDown(n) => {
                let col_first = self.first_col().unwrap();
                let col_total = self.col_count();
                if n >= col_total {
                    return false;
                }
                let new_col = col_first + n;
                if self.selected.col != new_col || self.piles[new_col].cards.len() < 2 {
                    self.selected = Pos { col: new_col, row: 0 };
                    return true;
                }
                if self.piles[self.selected.col].conf.take_only {
                    return false;
                }
                if self.selected.row == 0 {
                    let mx = self.first_up(new_col);
                    let new_pos = Some(Pos { col: self.selected.col, row: mx });
                    if !self.is_selectable(new_pos) {
                        return false;
                    }
                    self.selected = Pos { col: new_col, row: mx };
                } else {
                    self.selected = Pos { col: new_col, row: self.selected.row - 1 };
                }
            }
            Direction::Waste => {
                let count = self.fnd_count();
                let first = self.first_fnd().unwrap();
                self.move_next_pile_of_type(first, count);
            }
            Direction::Temp => {
                let count = self.temp_count();
                if count == 0 {
                    return false;
                }
                let first = self.first_temp().unwrap();
                self.move_next_pile_of_type(first, count);
            }
            Direction::Pile => {
                let count = self.pile_count();
                if count == 0 {
                    return false;
                }
                let first = self.first_pile().unwrap();
                self.move_next_pile_of_type(first, count);
            }
        }
        true
    }

    // return all cards that can be moved to another location
    pub fn avail_list(&self) -> Vec<Pos> {
        let mut loc: Vec<Pos> = Vec::new();
        let first_fnd = self.first_fnd().unwrap();
        let fnd_len = self.fnd_count();
        for (idx, pile) in self.piles.iter().enumerate() {
            if idx >= first_fnd && idx < first_fnd + fnd_len {
                continue;
            }
            let l = pile.cards.len();
            if l == 0 {
                continue;
            }
            let ordered = self.ordered_count(idx);
            for (cidx, card) in pile.cards.iter().enumerate() {
                if (pile.conf.take_only || pile.conf.playable == Playable::Top) && cidx < l - 1 {
                    continue;
                }
                if pile.conf.playable == Playable::Ordered && l - cidx > ordered {
                    continue;
                }
                if !card.is_up() {
                    continue;
                }
                let cpos = Pos { col: idx, row: l - cidx - 1 };
                let where_to = self.dest_list_card(cpos);
                if !where_to.is_empty() {
                    loc.push(cpos);
                }
            }
        }
        loc
    }

    // return all locations on which the card can be put.
    // In the following order: foundations, columns, free cells.
    pub fn dest_list_card(&self, from: Pos) -> Vec<Pos> {
        let card = self.card_at(from);
        let mut dests = Vec::new();
        if card.is_empty() {
            return dests;
        }
        for idx in 0..self.piles.len() {
            if idx == from.col {
                continue;
            }
            if self.can_move(from, idx) {
                dests.push(Pos { col: idx, row: 0 });
            }
        }
        dests
    }

    // rollback the game to the previous snapshot if exists. The snapshot is deleted, so redo is
    // unavailable.
    pub fn undo(&mut self) {
        if self.undo.is_empty() {
            return;
        }
        let mut last = self.undo.pop().unwrap(); // TODO
        self.redeals = last.redeals;
        self.selected = last.selected;
        for (idx, pile) in last.piles.drain(..).enumerate() {
            self.piles[idx].cards = pile;
        }
    }

    // empty the list of game snapshots, e.g. after winning the game
    pub fn clear_undo(&mut self) {
        self.undo.clear();
    }

    // the number of saved snapshots
    pub fn undo_count(&self) -> usize {
        self.undo.len()
    }

    // create a game snapshot
    pub fn take_snapshot(&mut self) {
        let mut undo = Undo { redeals: self.redeals, piles: Vec::new(), selected: self.selected };
        for pile in self.piles.iter() {
            undo.piles.push(pile.cards.clone());
        }
        self.undo.push(undo);
    }

    // compare two last game snapshots. If they equal, the latest one is removed
    pub fn squash_snapshots(&mut self) {
        let l = self.undo.len();
        if l < 2 {
            return;
        }
        let last = &self.undo[l - 1];
        let prev = &self.undo[l - 2];
        if last.redeals != prev.redeals {
            return;
        }
        for (idx, pile) in last.piles.iter().enumerate() {
            if pile.len() != prev.piles[idx].len() {
                return;
            }
        }
        self.undo.pop();
    }

    // move a card(or a few ones if the card is not at the top of its pile), if possible.
    // If "from" is empty, the currently marked or selected card is moved.
    // If "to" is empty, the card is moved to the first suitable location, if exists.
    pub fn move_card(&mut self, from: Pos, to: Pos) -> Result<(), SolError> {
        let from = if from.is_empty() {
            if self.selected.is_empty() {
                return Err(SolError::NotSelected);
            }
            self.selected
        } else {
            from
        };
        let src_card = self.card_at(from);
        if src_card.is_empty() {
            return Err(SolError::InvalidMove);
        }
        if self.piles[from.col].conf.playable == Playable::Top && from.row != 0 {
            return Err(SolError::Unplayable);
        }
        let to = if to.is_empty() {
            let dests = self.dest_list_card(from);
            if !dests.is_empty() {
                dests[0]
            } else {
                to
            }
        } else {
            to
        };
        if to.is_empty() || to.row != 0 {
            info!("nowhere to put {:?}", src_card);
            return Err(SolError::NoDestination);
        }
        if !self.can_move(from, to.col) {
            return Err(SolError::InvalidMove);
        }

        // all checks are done, moving the card
        let cnt = from.row + 1;
        let flippable = self.piles[from.col].conf.flip;
        let cfrom = &mut self.piles[from.col].cards;
        let l = cfrom.len();

        let mut to_move: CardList = Vec::new();
        for item in cfrom.iter().skip(l - cnt) {
            to_move.push(*item);
        }
        cfrom.truncate(l - cnt);
        let l = cfrom.len();
        if l != 0 {
            let crd = cfrom[l - 1];
            if !crd.is_up() && flippable {
                let mut c = cfrom.pop().unwrap();
                c.up = true;
                cfrom.push(c);
            }
        }
        for c in to_move.drain(..) {
            self.piles[to.col].cards.push(c);
        }
        Ok(())
    }

    // return a card at a given position or empty card if position is invalid.
    fn card_at(&self, loc: Pos) -> Card {
        if loc.is_empty() || loc.col >= self.piles.len() {
            return Card::new_empty();
        }
        let l = self.piles[loc.col].cards.len();
        if loc.row >= l {
            return Card::new_empty();
        }
        self.piles[loc.col].cards[l - 1 - loc.row]
    }

    // detect a solitaire win.
    pub fn is_completed(&self) -> bool {
        let wl = self.first_fnd().unwrap();
        let wc = self.fnd_count();
        for (i, p) in self.piles.iter().enumerate() {
            if i >= wl && i < wl + wc {
                continue;
            }
            if !p.cards.is_empty() {
                return false;
            }
        }
        true
    }

    // returns true is any game snapshot exists
    pub fn has_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    // return the number of foundation piles
    pub fn fnd_count(&self) -> usize {
        self.conf.fnd.len()
    }

    // return the number of free-cell piles
    pub fn temp_count(&self) -> usize {
        match self.conf.temp {
            None => 0,
            Some(ref cfg) => usize::from(cfg.count),
        }
    }

    // return the number of columns
    pub fn col_count(&self) -> usize {
        self.conf.cols.len()
    }

    // return the number of deck+waste
    pub fn pile_count(&self) -> usize {
        if let Some(ref pconf) = self.conf.pile {
            if pconf.pile_to_cols {
                1
            } else {
                2
            }
        } else {
            0
        }
    }

    // return a pile's configuration
    pub fn slot_conf(&self, pos: usize) -> Result<&SlotConf, SolError> {
        if pos >= self.piles.len() {
            return Err(SolError::InvalidLocation);
        }
        Ok(&self.piles[pos].conf)
    }

    // return cards in a deck(0) or in a waste(1)
    pub fn pile(&self, pos: usize) -> Result<&CardList, SolError> {
        if pos >= self.pile_count() {
            return Err(SolError::InvalidLocation);
        }
        let pp = self.first_pile().unwrap() + pos;
        Ok(&self.piles[pp].cards)
    }

    // return cards in a given foundation pile
    pub fn fnd(&self, pos: usize) -> Result<&CardList, SolError> {
        if pos >= self.fnd_count() {
            return Err(SolError::InvalidLocation);
        }
        let wp = self.first_fnd().unwrap() + pos;
        Ok(&self.piles[wp].cards)
    }

    // return cards in a given free-cell pile
    pub fn temp(&self, pos: usize) -> Result<&CardList, SolError> {
        if pos >= self.temp_count() {
            return Err(SolError::InvalidLocation);
        }
        let tp = self.first_temp().unwrap() + pos;
        Ok(&self.piles[tp].cards)
    }

    // return cards in a given column
    pub fn col(&self, pos: usize) -> Result<&CardList, SolError> {
        if pos >= self.col_count() {
            return Err(SolError::InvalidLocation);
        }
        let cp = self.first_col().unwrap() + pos;
        Ok(&self.piles[cp].cards)
    }

    // initialize a list of card in a pile. The pile contains exactly "count" cards and "up" of
    // them are face-up.
    fn gen_list(&mut self, count: usize, up: usize) -> Result<CardList, SolError> {
        let mut col = Vec::new();
        for i in 0..count {
            match self.deck.deal_card() {
                None => return Err(SolError::InsufficientFor("play area".to_string())),
                Some(mut crd) => {
                    if count - i <= up {
                        crd.up = true;
                    }
                    col.push(crd);
                }
            }
        }
        Ok(col)
    }

    // define all piles configurations(rules) at the solitaire initialization
    fn init_piles(&mut self) {
        // order: fnd, cols, temp, pile
        for idx in 0..self.fnd_count() {
            let mut conf = SlotConf::new_for_temp();
            let wc = &self.conf.fnd[idx];
            conf.start_face = wc.first;
            conf.start_suit = wc.suit;
            conf.suit_order = wc.sorder;
            conf.face_order = wc.forder;
            let p = Pile { conf, cards: Vec::new() };
            self.piles.push(p);
        }
        for i in 0..self.col_count() {
            let mut conf = SlotConf::new_for_col();
            conf.playable = self.conf.playable;
            conf.face_order = self.conf.col_forder;
            conf.suit_order = self.conf.col_sorder;
            conf.refill = self.conf.col_refill != Face::Empty && self.conf.col_refill != Face::Unavail;
            conf.start_face = self.conf.col_refill;
            conf.take_only = conf.take_only || self.conf.cols[i].take_only;
            let p = Pile { conf, cards: Vec::new() };
            self.piles.push(p);
        }
        for _i in 0..self.temp_count() {
            let mut conf = SlotConf::new_for_temp();
            conf.take_only = false;
            conf.refill = true;
            let p = Pile { conf, cards: Vec::new() };
            self.piles.push(p);
        }
        for idx in 0..self.pile_count() {
            let mut conf = SlotConf::new_for_pile();
            conf.selectable = idx == 1;
            conf.all_up = idx == 1;
            let p = Pile { conf, cards: Vec::new() };
            self.piles.push(p);
        }
    }

    // deal cards to all piles at the solitaire initialization
    pub fn init_cols(&mut self) -> Result<(), SolError> {
        self.init_piles();
        let idx = self.first_col().unwrap();
        let ccols = self.conf.cols.clone();
        for (n, cfg) in ccols.iter().enumerate() {
            let cnt = cfg.count;
            let up = if cfg.up == 0 && cnt != 0 {
                1
            } else if cfg.up > cfg.count {
                cfg.count
            } else {
                cfg.up
            };
            let col = self.gen_list(usize::from(cnt), usize::from(up))?;
            self.piles[idx + n].cards = col;
        }

        if let Some(tc) = self.conf.temp {
            let idx = self.first_temp().unwrap();
            let tcnt = usize::from(tc.count);
            for i in 0..tcnt {
                self.piles[idx + i].cards = Vec::new();
            }
        }

        if self.conf.pile.is_none() && !self.deck.is_empty() {
            return Err(SolError::UnusedCards);
        }
        if let Some(pconf) = self.conf.pile {
            let idx = self.first_pile().unwrap();
            while let Some(crd) = self.deck.deal_card() {
                self.piles[idx].cards.push(crd);
            }
            let mut cnt = pconf.deal_by;
            while !self.piles[idx].cards.is_empty() && !pconf.pile_to_cols && cnt != 0 {
                let mut crd = self.piles[idx].cards.pop().unwrap();
                crd.up = true;
                self.piles[idx + 1].cards.push(crd);
                cnt -= 1;
            }
        }
        Ok(())
    }

    // select a card at location `loc`. Returns false is the card cannot be
    // selected(e.g, faced down or empty/disabled pile)
    pub fn select_loc(&mut self, loc: Pos) -> bool {
        if loc == self.selected {
            return false;
        }
        if loc.col >= self.piles.len() {
            return false;
        }
        let cfg = &self.piles[loc.col].conf;
        if cfg.playable == Playable::Top && loc.row != 0 {
            return false;
        }
        if loc.row >= self.piles[loc.col].cards.len() {
            return false;
        }
        let crd = &self.piles[loc.col].cards[loc.row];
        crd.up
    }

    // return true if "deck" is playable: either the deck is non-empty or the waste is non-empty
    // and the number of redeals is greater than zero.
    pub fn can_deal(&self) -> bool {
        if self.pile_count() == 0 {
            return false;
        }
        let idx = self.first_pile().unwrap();
        if self.pile_count() == 1 {
            return !self.piles[idx].cards.is_empty();
        }
        !self.piles[idx].cards.is_empty() || self.redeals != 0
    }

    // deal one or a few cards from "deck" to "waste" if it exists, to directly to columns.
    // When dealing to columns, it always put one new card to the top of each column.
    pub fn deal(&mut self) -> bool {
        if self.pile_count() == 0 {
            return false;
        }
        let idx = self.first_pile().unwrap();
        if self.piles[idx].cards.is_empty() && self.redeals == 0 {
            return false;
        }

        if let Some(ref pconf) = self.conf.pile {
            // deal to columns
            if pconf.pile_to_cols {
                let col_first = self.first_col().unwrap();
                for col_idx in 0..self.col_count() {
                    if self.piles[col_idx + col_first].conf.take_only {
                        continue;
                    }
                    if self.piles[idx].cards.is_empty() {
                        break;
                    }
                    let mut crd = self.piles[idx].cards.pop().unwrap();
                    crd.up = true;
                    self.piles[col_idx + col_first].cards.push(crd);
                }
                return true;
            }
        }

        // if the deck is empty, move cards from the "waste" to the "deck" at first
        if self.piles[idx].cards.is_empty() && !self.piles[idx + 1].cards.is_empty() {
            self.redeals -= 1;
            while !self.piles[idx + 1].cards.is_empty() {
                let mut card = self.piles[idx + 1].cards.pop().unwrap();
                card.up = false;
                self.piles[idx].cards.push(card);
            }
        }
        if self.piles[idx].cards.is_empty() {
            return false;
        }
        // put a few top cards to the "waste"
        let mut cnt = self.conf.deal_by();
        while !self.piles[idx].cards.is_empty() && cnt != 0 {
            let mut crd = self.piles[idx].cards.pop().unwrap();
            crd.up = true;
            self.piles[idx + 1].cards.push(crd);
            cnt -= 1;
        }
        true
    }
}
