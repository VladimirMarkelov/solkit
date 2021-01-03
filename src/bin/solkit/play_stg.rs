use std::collections::HashMap;

use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEvent};
use crossterm::terminal;

use solkit::card::{Card, Suit};
use solkit::engine::{Direction, Game, Pos};
use solkit::err::SolError;
use solkit::gconf::{self, FaceOrder, Playable, SuitOrder};

use crate::primitive::{Border, Screen};
use crate::strategy::{Context, Strategy, Transition, TransitionStage};
use crate::theme::Theme;
use crate::ui::{area_width, draw_area, draw_suit, face_to_str};

const FND_STR: &str = "foundation";
const FRC_STR: &str = "free cell";
const DCK_STR: &str = "deck";
const WST_STR: &str = "waste";
const COL_STR: &str = "column";

fn col_to_string(game: &Game, col: usize) -> &'static str {
    let cnt = game.pile_count();
    if cnt != 0 {
        let first = game.first_pile().expect("first pile must exist");
        if col == first {
            return DCK_STR;
        }
        if col == first + 1 && cnt == 2 {
            return WST_STR;
        }
    }
    let cnt = game.temp_count();
    if cnt != 0 {
        let first = game.first_temp().expect("first temp must exist");
        if col >= first && col < first + cnt {
            return FRC_STR;
        }
    }
    let cnt = game.fnd_count();
    let first = game.first_fnd().expect("foundation must exist");
    if col >= first && col < first + cnt {
        return FND_STR;
    }
    let cnt = game.col_count();
    let first = game.first_col().expect("column must exist");
    if col >= first && col < first + cnt {
        return COL_STR;
    }
    ""
}

// return true if a slot is a column or foundation (i.e, for which face and suit order makes sense)
fn slot_is_playable(game: &Game, col: usize) -> bool {
    let cnt = game.fnd_count();
    let first = game.first_fnd().expect("foundation must exist");
    if col >= first && col < first + cnt {
        return true;
    }
    let cnt = game.col_count();
    let first = game.first_col().expect("column must exist");
    if col >= first && col < first + cnt {
        return true;
    }
    false
}

// main "dialog" - playing a solitaire
pub(crate) struct PlayStg<'a> {
    game: Game<'a>,
}

impl<'a> PlayStg<'a> {
    pub(crate) fn new(rules: &'a HashMap<String, gconf::Conf>, ctx: &mut Context) -> Result<Self, SolError> {
        let gc = match rules.get(&ctx.name) {
            None => return Err(SolError::SolitaireNotExist(ctx.name.to_string())),
            Some(rule) => rule,
        };
        let game = Game::init(gc)?;
        Ok(PlayStg { game })
    }

    fn draw_stats(&self, ctx: &mut Context, scr: &mut Screen, theme: &dyn Theme) {
        let (fg, bg) = theme.base_colors();
        let area_w = area_width(&self.game);
        let x = area_w + 2;
        let mut y = 1;
        scr.colors(fg, bg);
        scr.write_string(&ctx.name, x, y);
        y += 2;

        let stats = ctx.stats.game_stat(&ctx.name);
        let played = if ctx.moved { stats.played + 1 } else { stats.played };
        let won = if ctx.won { stats.won + 1 } else { stats.won };
        let prc = if played == 0 { 0.0f32 } else { won as f32 / played as f32 };
        let msg = format!("{:7}{:>7}", "Played:", played);
        scr.write_string(&msg, x, y);
        y += 1;
        let msg = format!("{:7}{:>7}", "Won:", won);
        scr.write_string(&msg, x, y);
        y += 1;
        let msg = format!("{:7}{:>7.1}", "%", prc);
        scr.write_string(&msg, x, y);
        if self.game.pile_count() == 2 {
            if self.game.redeal_left() >= 0 {
                let msg = format!("{:8}{:>6}", "Redeals:", self.game.redeal_left());
                scr.write_string(&msg, x, y);
            } else {
                let msg = format!("{:8}{:>6}", "Redeals:", "∞");
                scr.write_string(&msg, x, y);
            }
            y += 1;
        }
        y += 2;

        let sloc = self.game.selected_loc();
        let curr_pile = format!("Current pile: {}", col_to_string(&self.game, sloc.col));
        scr.write_string(&curr_pile, x, y);
        y += 1;
        let ccnt = match self.game.slot_cards_left(sloc.col) {
            Ok(left) => format!("cards: {}", left),
            Err(_) => "cards: ?".to_string(),
        };
        scr.write_string(&ccnt, x, y);
        y += 1;
        let is_playable = slot_is_playable(&self.game, sloc.col);
        let cfg = self.game.slot_conf(sloc.col).expect("pile must have configuration");
        let fsort = if is_playable && cfg.suit_order != SuitOrder::Forbid && !cfg.take_only {
            match cfg.face_order {
                FaceOrder::Asc => "A 2 3 4",
                FaceOrder::Desc => "4 3 2 A",
                FaceOrder::Any => "A 2 3 2",
            }
        } else {
            "-"
        };
        let sford = format!("face order: {}", fsort);
        scr.write_string(&sford, x, y);
        y += 1;
        let ssord = "suit order:";
        scr.write_string(ssord, x, y);
        let sorder: &[Suit] = match cfg.suit_order {
            SuitOrder::SameSuit => &[Suit::Heart, Suit::Heart, Suit::Heart, Suit::Heart],
            SuitOrder::SameColor => &[Suit::Heart, Suit::Diamond, Suit::Diamond, Suit::Heart],
            SuitOrder::AlternateColor => &[Suit::Heart, Suit::Spade, Suit::Diamond, Suit::Club],
            _ => &[],
        };
        let xsord = x + ssord.len() as u16;
        if sorder.is_empty() || cfg.take_only {
            scr.write_string("-", xsord + 1, y);
        } else {
            for (idx, s) in sorder.iter().enumerate() {
                draw_suit(*s, scr, xsord + 1 + (idx as u16) * 2, y, theme);
            }
        }
        y += 1;

        let splayble = match cfg.playable {
            _ if cfg.take_only => "top only",
            Playable::Top => "top only",
            Playable::Any => "any group",
            Playable::Ordered => "ordered group",
        };
        let splay = format!("playble: {}", splayble);
        scr.write_string(&splay, x, y);

        y += 2;
        let markcardtext = "Marked card:";
        let markcardpos = ctx.state.marked();
        let markcard = if markcardpos.is_empty() { Card::new_empty() } else { self.game.card_at(markcardpos) };
        scr.write_string(markcardtext, x, y);
        if markcardpos.is_empty() || markcard.is_empty() {
            scr.write_string("-", x + markcardtext.len() as u16 + 1, y);
        } else {
            let fc = face_to_str(markcard.face);
            scr.write_string(&fc, x + markcardtext.len() as u16 + 1, y);
            draw_suit(markcard.suit, scr, x + markcardtext.len() as u16 + 2 + fc.len() as u16, y, theme)
        }
        y += 1;

        let selcardtext = "Selected card:";
        let selcard = self.game.card_at(sloc);
        scr.write_string(selcardtext, x, y);
        if selcard.is_empty() || !selcard.is_up() {
            scr.write_string("-", x + selcardtext.len() as u16 + 1, y);
        } else {
            let fc = face_to_str(selcard.face);
            let shift = x + selcardtext.len() as u16 + 1;
            scr.write_string(&fc, shift, y);
            draw_suit(selcard.suit, scr, shift + fc.len() as u16 + 1, y, theme);
            if sloc == markcardpos {
                scr.write_string("[MARKED]", shift + fc.len() as u16 + 3, y);
            }
        }
    }
}

fn on_enter(pstg: &mut PlayStg, ctx: &mut Context, curr: Pos) {
    pstg.game.take_snapshot();
    ctx.state.clear_hints();
    if pstg.game.is_deck_clicked(Some(curr)) {
        ctx.moved = true;
        ctx.state.clear_mark();
        pstg.game.deal();
    }
    let sel = ctx.state.marked();
    if sel.is_empty() || sel == curr {
        if pstg.game.move_card(curr, Pos::new()).is_ok() {
            ctx.moved = true;
            pstg.game.select(Pos { col: pstg.game.selected_loc().col, row: 0 });
            ctx.state.clear_mark();
        }
    } else if pstg.game.move_card(sel, curr).is_ok() {
        ctx.moved = true;
        pstg.game.select(Pos { col: pstg.game.selected_loc().col, row: 0 });
        ctx.state.clear_mark();
    }
    if pstg.game.is_completed() {
        pstg.game.clear_undo();
        ctx.won = true;
    } else {
        pstg.game.squash_snapshots();
    }
}

fn on_deal(pstg: &mut PlayStg, ctx: &mut Context) {
    ctx.moved = true;
    pstg.game.take_snapshot();
    ctx.state.clear_mark();
    pstg.game.deal();
    pstg.game.squash_snapshots();
}

impl<'a> Strategy for PlayStg<'a> {
    fn process_event(&mut self, ctx: &mut Context, scr: &mut Screen, event: Event) -> Result<Transition, SolError> {
        match event {
            Event::Key(ev) => match ev.code {
                KeyCode::Esc => {
                    return Ok(Transition::Push(TransitionStage::EndDialog));
                }
                KeyCode::Char('q') => {
                    if ev.modifiers == KeyModifiers::CONTROL {
                        return Ok(Transition::Exit);
                    } else {
                        return Ok(Transition::Push(TransitionStage::EndDialog));
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    let _changed = self.game.move_selection(Direction::Left);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    let _changed = self.game.move_selection(Direction::Right);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let _changed = self.game.move_selection(Direction::Up);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let _changed = self.game.move_selection(Direction::Down);
                }
                KeyCode::Char('1') => {
                    let _changed = self.game.move_selection(Direction::ColUp(0));
                }
                KeyCode::Char('2') => {
                    let _changed = self.game.move_selection(Direction::ColUp(1));
                }
                KeyCode::Char('3') => {
                    let _changed = self.game.move_selection(Direction::ColUp(2));
                }
                KeyCode::Char('4') => {
                    let _changed = self.game.move_selection(Direction::ColUp(3));
                }
                KeyCode::Char('5') => {
                    let _changed = self.game.move_selection(Direction::ColUp(4));
                }
                KeyCode::Char('6') => {
                    let _changed = self.game.move_selection(Direction::ColUp(5));
                }
                KeyCode::Char('7') => {
                    let _changed = self.game.move_selection(Direction::ColUp(6));
                }
                KeyCode::Char('8') => {
                    let _changed = self.game.move_selection(Direction::ColUp(7));
                }
                KeyCode::Char('9') => {
                    let _changed = self.game.move_selection(Direction::ColUp(8));
                }
                KeyCode::Char('0') => {
                    let _changed = self.game.move_selection(Direction::ColUp(9));
                }
                KeyCode::Char('!') => {
                    let _changed = self.game.move_selection(Direction::ColDown(0));
                }
                KeyCode::Char('@') => {
                    let _changed = self.game.move_selection(Direction::ColDown(1));
                }
                KeyCode::Char('#') => {
                    let _changed = self.game.move_selection(Direction::ColDown(2));
                }
                KeyCode::Char('$') => {
                    let _changed = self.game.move_selection(Direction::ColDown(3));
                }
                KeyCode::Char('%') => {
                    let _changed = self.game.move_selection(Direction::ColDown(4));
                }
                KeyCode::Char('^') => {
                    let _changed = self.game.move_selection(Direction::ColDown(5));
                }
                KeyCode::Char('&') => {
                    let _changed = self.game.move_selection(Direction::ColDown(6));
                }
                KeyCode::Char('*') => {
                    let _changed = self.game.move_selection(Direction::ColDown(7));
                }
                KeyCode::Char('(') => {
                    let _changed = self.game.move_selection(Direction::ColDown(8));
                }
                KeyCode::Char(')') => {
                    let _changed = self.game.move_selection(Direction::ColDown(9));
                }
                KeyCode::Char('f') => {
                    let _changed = self.game.move_selection(Direction::Waste);
                }
                KeyCode::Char('c') => {
                    let _changed = self.game.move_selection(Direction::Temp);
                }
                KeyCode::Char('d') => {
                    let _changed = self.game.move_selection(Direction::Pile);
                }

                KeyCode::Char(' ') => {
                    if ctx.won {
                        return Ok(Transition::None);
                    }
                    ctx.state.clear_hints();
                    if self.game.is_selectable(None) {
                        ctx.state.mark(self.game.selected_loc());
                    } else if self.game.is_deck_clicked(None) {
                        on_deal(self, ctx);
                    }
                }

                KeyCode::Enter | KeyCode::Char('m') => {
                    let pos = self.game.selected_loc();
                    on_enter(self, ctx, pos);
                }

                KeyCode::F(5) | KeyCode::Char('R') => {
                    return Ok(Transition::Replace(TransitionStage::Play));
                }
                KeyCode::F(1) => {
                    return Ok(Transition::Push(TransitionStage::HelpDialog));
                }

                KeyCode::Char('s') => {
                    ctx.state.hint(&self.game.avail_list());
                }

                KeyCode::Char('S') => {
                    ctx.state.hint(&self.game.dest_list_card(self.game.selected_loc()));
                }

                KeyCode::Char('u') => {
                    self.game.undo();
                }

                _ => {}
            },
            Event::Mouse(ev) => {
                if let MouseEvent::Down(btn, x, y, _) = ev {
                    match btn {
                        MouseButton::Left => {
                            // TODO: merge with SPACE
                            if ctx.won {
                                return Ok(Transition::None);
                            }
                            let w = scr.what_at(x, y);
                            if w == 0 {
                                return Ok(Transition::None);
                            }
                            let p = Pos { col: (w / 100) as usize, row: (w % 100) as usize };
                            ctx.state.clear_hints();
                            if self.game.is_selectable(Some(p)) {
                                self.game.select(p);
                                ctx.state.mark(p);
                            } else if self.game.is_deck_clicked(Some(p)) {
                                on_deal(self, ctx);
                            }
                        }
                        MouseButton::Right => {
                            if ctx.won {
                                return Ok(Transition::None);
                            }
                            let w = scr.what_at(x, y);
                            if w == 0 {
                                return Ok(Transition::None);
                            }
                            let p = Pos { col: (w / 100) as usize, row: (w % 100) as usize };
                            on_enter(self, ctx, p);
                        }
                        _ => {}
                    }
                }
            }
            Event::Resize(_, _) => {
                let (width, height) = match terminal::size() {
                    Err(e) => return Err(SolError::Unexpected(format!("{:?}", e))),
                    Ok((ww, hh)) => (ww, hh),
                };
                if width < 60 || height < 25 {
                    return Err(SolError::InvalidTermSize(width, height));
                }
                if let Err(e) = scr.resize(width, height) {
                    return Err(SolError::Unexpected(format!("Failed to resize: {:?}", e)));
                }
                ctx.w = width;
                ctx.h = height;
            }
        }
        Ok(Transition::None)
    }

    fn draw(&self, ctx: &mut Context, scr: &mut Screen, theme: &dyn Theme) -> Result<(), SolError> {
        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        scr.clear();
        draw_area(scr, &self.game, &ctx.state, theme)?;
        self.draw_stats(ctx, scr, theme);

        if ctx.won {
            const VICTORY_MSG: &str = "You win!";
            let vlen = VICTORY_MSG.len() as u16;
            let x = ctx.w / 2 - vlen / 2;
            let y = ctx.h / 2 - 1;

            scr.draw_frame(x - 3, y - 2, vlen + 6, 5, Border::Double);
            scr.fill_rect(x - 2, y - 1, vlen + 4, 3, ' ');
            let (fg, bg) = theme.win_msg();
            scr.colors(fg, bg);
            scr.write_string(VICTORY_MSG, x, y);
        }

        Ok(())
    }
}
