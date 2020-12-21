use std::cmp;

use solkit::card::{Card, Face, Suit};
use solkit::engine::{Game, Pos, SlotConf};
use solkit::err::SolError;

use crate::gstate::GameState;
use crate::primitive::{Border, Screen};
use crate::theme::Theme;

// const CARD_BACK: char = '✿';//'◊';//'/';//'░';
const CARD_HEIGHT: u16 = 5;
const CARD_WIDTH: u16 = 6;

bitflags! {
    #[derive(Default)]
    pub struct CardState: u32 {
        const CURRENT = 1;
        const SELECTED = 2;
        const HINT = 4;
        const SQUASH = 8;
        const EMPTY = 16;
        const FORBIDDEN = 32;
    }
}

enum DrawPile {
    Normal,
    SquashDown,
    SquashAll,
    HideDown,
}

fn suit_to_char(s: Suit) -> char {
    match s {
        Suit::Spade => '♠', //'♤',//'♠',
        Suit::Club => '♣',  //'♧',//'♣',
        Suit::Diamond => '♦',
        Suit::Heart => '♥',
        _ => ' ',
    }
}

fn face_to_str(f: Face) -> &'static str {
    match f {
        Face::N2 => "2",
        Face::N3 => "3",
        Face::N4 => "4",
        Face::N5 => "5",
        Face::N6 => "6",
        Face::N7 => "7",
        Face::N8 => "8",
        Face::N9 => "9",
        Face::N10 => "10",
        Face::J => "J",
        Face::Q => "Q",
        Face::K => "K",
        Face::A => "A",
        Face::Any => "?",
        _ => "",
    }
}

fn pile_draw_style(cards: &[Card], max_height: u16) -> DrawPile {
    let max_height = usize::from(max_height);
    let n = cards.len() * 2;
    if n <= max_height {
        return DrawPile::Normal;
    }
    let mut down = 0;
    for c in cards {
        if !c.is_up() {
            down += 1;
        }
    }
    let up = cards.len() - down;
    if up * 2 + down <= max_height {
        return DrawPile::SquashDown;
    }
    if up + down <= max_height {
        return DrawPile::SquashAll;
    }
    DrawPile::HideDown
}

// Pile that cannot be redealt/refilled.
// Kind must be set for Screen before calling the function.
fn draw_forbidden_card(scr: &mut Screen, col: u16, row: u16, border: Border, theme: &dyn Theme) {
    let (fg, bg) = theme.forbidden_card();
    scr.colors(fg, bg);
    scr.draw_frame(col, row, CARD_WIDTH, CARD_HEIGHT, border);
    scr.fill_rect(col + 1, row + 1, CARD_WIDTH - 2, CARD_HEIGHT - 2, ' ');
    let (fg, bg) = theme.forbidden_area();
    scr.colors(fg, bg);
    scr.write_char('x', col + 1, row + 1);
    scr.write_char('x', col + 4, row + 1);
    scr.write_string("xx", col + 2, row + 2);
    scr.write_char('x', col + 1, row + CARD_HEIGHT - 2);
    scr.write_char('x', col + 4, row + CARD_HEIGHT - 2);
}

// Card face down.
// Kind must be set for Screen before calling the function
fn draw_card_down(scr: &mut Screen, col: u16, row: u16, border: Border, theme: &dyn Theme) {
    let (fg, bg) = theme.base_colors();
    scr.colors(fg, bg);
    scr.draw_frame(col, row, CARD_WIDTH, CARD_HEIGHT, border);
    let (fg, bg) = theme.card_back();
    scr.colors(fg, bg);
    for y in row + 1..row + 1 + CARD_HEIGHT - 2 {
        for x in col + 1..col + 1 + CARD_WIDTH - 2 {
            let c = if (x - col - 1) % 2 == 0 { '▀' } else { '▄' };
            scr.write_char(c, x, y)
        }
    }
}

// Empty space that can be filled with something.
// Kind must be set for Screen before calling the function
fn draw_empty_card(scr: &mut Screen, card: Card, col: u16, row: u16, border: Border, theme: &dyn Theme) {
    let (fg, bg) = theme.empty_card();
    scr.colors(fg, bg);
    scr.draw_frame(col, row, CARD_WIDTH, CARD_HEIGHT, border);
    scr.fill_rect(col + 1, row + 1, CARD_WIDTH - 2, CARD_HEIGHT - 2, ' ');
    let fstr = face_to_str(card.face);
    let shift = if card.suit == Suit::Any { 2 } else { 1 };
    scr.write_string(fstr, col + shift, row + 2);
    scr.write_char('╌', col + 1, row);
    scr.write_char('╌', col + 4, row);
    scr.write_char('╌', col + 1, row + CARD_HEIGHT - 1);
    scr.write_char('╌', col + 4, row + CARD_HEIGHT - 1);
    scr.write_char('╎', col, row + 1);
    scr.write_char('╎', col + CARD_WIDTH - 1, row + 1);
    scr.write_char('╎', col, row + 3);
    scr.write_char('╎', col + CARD_WIDTH - 1, row + 3);
    if card.suit == Suit::Any {
        let sch = suit_to_char(Suit::Heart);
        scr.write_char(sch, col + 1, row + 1);
        let sch = suit_to_char(Suit::Spade);
        scr.write_char(sch, col + CARD_WIDTH - 2, row + 1);
        let sch = suit_to_char(Suit::Club);
        scr.write_char(sch, col + 1, row + CARD_HEIGHT - 2);
        let sch = suit_to_char(Suit::Diamond);
        scr.write_char(sch, col + CARD_WIDTH - 2, row + CARD_HEIGHT - 2);
    } else {
        let sch = suit_to_char(card.suit);
        scr.write_char(sch, col + 4, row + 2);
    }
}

pub(crate) fn draw_card(
    scr: &mut Screen,
    col: u16,
    row: u16,
    card: Card,
    kind: u16,
    flags: CardState,
    theme: &dyn Theme,
) {
    let border = if flags.contains(CardState::CURRENT) { Border::Double } else { Border::Single };
    scr.kind(kind);
    if flags.contains(CardState::FORBIDDEN) {
        draw_forbidden_card(scr, col, row, border, theme);
        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        return;
    }
    if flags.contains(CardState::EMPTY) {
        draw_empty_card(scr, card, col, row, border, theme);
        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        return;
    }
    if !card.is_up() {
        draw_card_down(scr, col, row, border, theme);
        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        return;
    }
    let (fg, bg) = if flags.contains(CardState::SELECTED) {
        theme.selected_card()
    } else if flags.contains(CardState::HINT) {
        theme.hint_card()
    } else {
        theme.card()
    };
    scr.colors(fg, bg);
    scr.draw_frame(col, row, CARD_WIDTH, CARD_HEIGHT, border);
    let (fg, bg) = theme.card();
    scr.colors(fg, bg);
    scr.fill_rect(col + 1, row + 1, CARD_WIDTH - 2, CARD_HEIGHT - 2, ' ');

    let fy = if flags.contains(CardState::SQUASH) { row } else { row + 1 };
    let fstr = face_to_str(card.face);
    scr.write_string(fstr, col + 1, fy);
    let last_row = row + CARD_HEIGHT - 2;
    scr.write_string(fstr, col + 5 - fstr.len() as u16, last_row);
    scr.colors(theme.suit(card.suit), bg);
    let sch = suit_to_char(card.suit);
    scr.write_char(sch, col + 4, fy);
    scr.write_char(sch, col + 1, last_row);
    let (fg, bg) = theme.base_colors();
    scr.colors(fg, bg);
}

fn is_in_list(val: Pos, list: &[Pos]) -> bool {
    for lv in list {
        if *lv == val {
            return true;
        }
    }
    false
}

#[derive(Copy, Clone)]
pub struct ScrPos {
    pub col: u16,
    pub row: u16,
}
#[derive(Copy, Clone)]
pub struct PileProps<'a> {
    pub pile: &'a [Card],
    pub id: usize,
    pub conf: &'a SlotConf,
}
#[derive(Copy, Clone)]
pub struct DrawHints<'a> {
    selected: Pos,
    current: Pos,
    hinted: &'a [Pos],
}

pub(crate) fn draw_pile(
    scr: &mut Screen,
    scr_pos: ScrPos,
    max_height: u16,
    pile_props: PileProps,
    draw_hints: DrawHints,
    theme: &dyn Theme,
) {
    let (col, row) = (scr_pos.col, scr_pos.row); // TODO: use scr_pos directly
    let (pile, id, conf) = (pile_props.pile, pile_props.id, pile_props.conf); // TODO: use pile_props directly
    let (selected, current, hinted) = (draw_hints.selected, draw_hints.current, draw_hints.hinted); // TODO: use draw_hints directly
    if pile.is_empty() {
        let crd = Card::new(conf.start_suit, conf.start_face);
        let mut state = if !conf.refill { CardState::FORBIDDEN } else { CardState::EMPTY };
        if current.col == id {
            state |= CardState::CURRENT
        };
        draw_card(scr, col, row, crd, (id as u16) * 100, state, theme);
        return;
    }
    if !conf.selectable && !conf.all_up {
        // pile that always faces down
        let crd = Card::new_empty();
        let state = if current.col == id { CardState::CURRENT } else { CardState::empty() };
        draw_card(scr, col, row, crd, (id as u16) * 100, state, theme);
        return;
    }
    if !conf.draw_all {
        let crd = pile[pile.len() - 1];
        let crd_pos = Pos { col: id, row: 0 };
        let mut state = if crd_pos == current { CardState::CURRENT } else { CardState::empty() };
        if crd_pos == selected {
            state |= CardState::SELECTED;
        }
        if is_in_list(crd_pos, hinted) {
            state |= CardState::HINT;
        }
        draw_card(scr, col, row, crd, (id as u16) * 100, state, theme);
        return;
    }

    let style = pile_draw_style(pile, max_height);
    let mut down = 0;
    let mut up = 0;
    for c in pile {
        if c.is_up() {
            up += 1;
        } else {
            down += 1;
        }
    }
    let (dup, ddown) = match style {
        DrawPile::HideDown => (1u16, 0u16),
        DrawPile::SquashAll => (1, 1),
        DrawPile::SquashDown => (2, 1),
        DrawPile::Normal => (2, 2),
    };
    let mut dy = row;
    if down != 0 {
        if ddown == 0 {
            let crd = Card::new_empty();
            let cid = (pile.len() - up) as u16;
            draw_card(scr, col, row, crd, (id as u16) * 100 + cid, CardState::empty(), theme);
            if down > 1 {
                let cnt = format!("+{}", down - 1);
                scr.write_string(&cnt, col + 2, row);
            }
            dy += 1;
        } else {
            let l = (pile.len() - 1) as u16;
            let crd = Card::new_empty();
            for idx in 0..down {
                draw_card(scr, col, dy, crd, (id as u16) * 100 + l - idx as u16, CardState::empty(), theme);
                dy += ddown;
            }
        }
    }
    if up == 0 {
        return;
    }
    let l = pile.len();
    for idx in 0..up {
        let crd = pile[l + idx - up];
        let crd_pos = Pos { col: id, row: up - idx - 1 };
        let cid = (id * 100 + crd_pos.row) as u16;
        let mut state = if dup == 1 { CardState::SQUASH } else { CardState::empty() };
        if crd_pos == current {
            state |= CardState::CURRENT
        };
        if crd_pos.col == selected.col && crd_pos.row <= selected.row {
            state |= CardState::SELECTED;
        }
        if is_in_list(crd_pos, hinted) {
            state |= CardState::HINT;
        }
        draw_card(scr, col, dy, crd, cid, state, theme);
        dy += dup;
    }
}

pub(crate) fn area_width(game: &Game) -> u16 {
    let pile_cnt = game.pile_count();
    let fnd_cnt = game.fnd_count();
    let temp_cnt = game.temp_count();
    let col_cnt = game.col_count();

    let top_shift = if pile_cnt == 0 {
        1
    } else if pile_cnt == 2 {
        2 * CARD_WIDTH + 1 + CARD_WIDTH / 2 + 1
    } else {
        CARD_WIDTH + CARD_WIDTH / 2 + 1
    };
    let bottom_shift = if temp_cnt == 0 { 1 } else { CARD_WIDTH + CARD_WIDTH / 2 + 1 };

    let fnd_w = fnd_cnt as u16 * CARD_WIDTH + fnd_cnt as u16;
    let cols_w = col_cnt as u16 * CARD_WIDTH + col_cnt as u16;

    let top = fnd_w + top_shift;
    let bottom = cols_w + bottom_shift;

    cmp::max(top, bottom)
}

pub(crate) fn draw_area(scr: &mut Screen, game: &Game, state: &GameState, theme: &dyn Theme) -> Result<(), SolError> {
    scr.clear();
    let pile_cnt = game.pile_count();
    let temp_cnt = game.temp_count();
    let fnd_cnt = game.fnd_count();
    let col_cnt = game.col_count();
    let marked = state.marked(); // TODO: highlight the entire column
    let hints = state.hints();

    let (idx, idy) = (1u16, 0u16);

    // println!("pile...{}", pile_cnt);
    if pile_cnt != 0 {
        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        let crd = Card::new_empty();
        let id = game.first_pile().unwrap();
        let pile_pos = Pos { col: id, row: 0 };
        let mut state = if pile_pos == game.selected_loc() { CardState::CURRENT } else { CardState::empty() };
        if !game.can_deal() {
            state |= CardState::FORBIDDEN;
        } else {
            let lst = game.pile(0)?;
            if lst.is_empty() {
                state |= CardState::EMPTY
            }
        }
        draw_card(scr, idx, idy, crd, (id * 100) as u16, state, theme);
        if pile_cnt > 1 {
            let lst = game.pile(1)?;
            let cfg = game.slot_conf(id + 1)?;
            draw_pile(
                scr,
                ScrPos { col: idx + CARD_WIDTH + 1, row: idy },
                10, // pile height is always one card high
                PileProps { pile: lst, id: id + 1, conf: cfg },
                DrawHints { selected: marked, current: game.selected_loc(), hinted: &hints },
                theme,
            );
        }
        let (fg, bg) = theme.hint_letter();
        scr.colors(fg, bg);
        scr.kind(0);
        scr.write_char('d', idx - 1, idy);
    }

    // println!("temp...");
    for tidx in 0..temp_cnt {
        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        let fid = game.first_temp().unwrap();
        let y = idy + CARD_HEIGHT + 2 + tidx as u16 * CARD_HEIGHT;
        let cfg = game.slot_conf(fid + tidx)?;
        let lst = game.temp(tidx)?;
        draw_pile(
            scr,
            ScrPos { col: idx, row: y },
            10, // temp height is always one card high
            PileProps { pile: lst, id: fid + tidx, conf: cfg },
            DrawHints { selected: marked, current: game.selected_loc(), hinted: &hints },
            theme,
        );
        if tidx == 0 {
            let (fg, bg) = theme.hint_letter();
            scr.colors(fg, bg);
            scr.kind(0);
            scr.write_char('c', idx - 1, y);
        }
    }

    let xshift = if pile_cnt == 0 && temp_cnt == 0 {
        0
    } else if pile_cnt == 2 {
        2 * CARD_WIDTH + 1 + CARD_WIDTH / 2
    } else {
        CARD_WIDTH + CARD_WIDTH / 2
    };
    // println!("fnd...");
    for widx in 0..fnd_cnt {
        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        let fid = game.first_fnd().unwrap();
        let x = idx + xshift + widx as u16 * CARD_WIDTH + widx as u16;
        let cfg = game.slot_conf(fid + widx)?;
        let lst = game.fnd(widx)?;
        draw_pile(
            scr,
            ScrPos { col: x, row: idy },
            10, // fnd height is always is one card high
            PileProps { pile: lst, id: fid + widx, conf: cfg },
            DrawHints { selected: marked, current: game.selected_loc(), hinted: &hints },
            theme,
        );
        if widx == 0 {
            let (fg, bg) = theme.hint_letter();
            scr.colors(fg, bg);
            scr.colors(fg, bg);
            scr.kind(0);
            scr.write_char('f', x - 1, idy);
        }
    }

    // println!("col...");
    let xshift = if temp_cnt == 0 { 0 } else { CARD_WIDTH + CARD_WIDTH / 2 };
    for cidx in 0..col_cnt {
        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        let fid = game.first_col().unwrap();
        let y = idy + 2 + CARD_HEIGHT;
        let x = idx + xshift + cidx as u16 * CARD_WIDTH + cidx as u16;
        let cfg = game.slot_conf(fid + cidx)?;
        let lst = game.col(cidx)?;
        let max_height = scr.height() - y;
        draw_pile(
            scr,
            ScrPos { col: x, row: y },
            max_height,
            PileProps { pile: lst, id: fid + cidx, conf: cfg },
            DrawHints { selected: marked, current: game.selected_loc(), hinted: &hints },
            theme,
        );

        let ch = std::char::from_u32('1' as u32 + (cidx % 10) as u32).unwrap_or(' ');
        let (fg, bg) = theme.hint_letter();
        scr.colors(fg, bg);
        scr.kind(0);
        scr.write_char(ch, x, y - 1);
    }

    Ok(())
}
