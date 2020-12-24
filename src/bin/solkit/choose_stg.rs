use std::collections::HashMap;

use crossterm::event::{Event, KeyCode};
use crossterm::terminal;

use solkit::err::SolError;
use solkit::gconf::Conf;

use crate::primitive::{Border, Screen};
use crate::strategy::{Context, Strategy, Transition, TransitionStage};
use crate::theme::Theme;

// dialog to select a solitaire from a list
pub(crate) struct ChooseStg {
    sols: Vec<String>,
    selected: u16,
    top: u16,
    height: u16,
    width: u16,
}

impl ChooseStg {
    pub(crate) fn new(rules: &HashMap<String, Conf>, ctx: &mut Context) -> Result<Self, SolError> {
        let mut cs = ChooseStg { sols: Vec::new(), selected: 0, top: 0, height: 0, width: 0 };
        for (name, _cfg) in rules.iter() {
            cs.sols.push(name.clone());
        }
        if cs.sols.is_empty() {
            return Err(SolError::SolitaireListEmpty);
        }
        cs.sols.sort();
        if !ctx.name.is_empty() {
            for (idx, n) in cs.sols.iter().enumerate() {
                if n == &ctx.name {
                    cs.selected = idx as u16;
                    break;
                }
            }
        }
        cs.update_size(ctx.w, ctx.h);
        Ok(cs)
    }

    fn update_size(&mut self, w: u16, h: u16) {
        self.height = h - 6;
        self.width = w - 10;
        if self.selected < self.top {
            self.top = self.selected;
        } else if self.selected >= self.top + self.height {
            self.top = self.selected - self.height + 1;
        }
    }
}

fn shift_in(s: &str, w: u16) -> u16 {
    let md = s.len() as u16 / 2;
    let wmd = w / 2;
    if wmd < md {
        0
    } else {
        wmd - md
    }
}

impl Strategy for ChooseStg {
    fn process_event(&mut self, ctx: &mut Context, scr: &mut Screen, event: Event) -> Result<Transition, SolError> {
        let l = self.sols.len() as u16;
        if self.height == 0 {
            self.update_size(scr.width(), scr.height());
        }
        let list_h = self.height - 2;
        match event {
            Event::Key(ev) => match ev.code {
                KeyCode::Esc => return Ok(Transition::Exit),
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected != 0 {
                        self.selected -= 1;
                        if self.selected < self.top {
                            self.top = self.selected;
                        }
                    }
                }
                KeyCode::Home | KeyCode::Char('K') => {
                    self.selected = 0;
                    self.top = 0;
                }
                KeyCode::End | KeyCode::Char('J') => {
                    self.selected += list_h;
                    if self.selected >= l {
                        self.selected = l - 1;
                    }
                    if self.selected >= self.top + list_h {
                        self.top = l - list_h;
                    }
                }
                KeyCode::PageUp | KeyCode::Char('u') => {
                    if self.top < list_h {
                        self.selected = 0;
                        self.top = 0;
                    } else {
                        self.top -= list_h;
                        self.selected -= list_h;
                    }
                }
                KeyCode::PageDown | KeyCode::Char('d') => {
                    if self.selected + list_h >= l {
                        self.top = if l > list_h { l - list_h } else { 0 };
                        self.selected = l - 1;
                    } else {
                        self.top += list_h;
                        self.selected += list_h;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected != l - 1 {
                        if self.selected - self.top >= list_h {
                            self.top += 1;
                        }
                        self.selected += 1;
                    }
                }

                KeyCode::Enter => {
                    ctx.name = self.sols[self.selected as usize].clone();
                    return Ok(Transition::Replace(TransitionStage::Play));
                }

                _ => {}
            },
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
                self.update_size(width, height);
            }
            _ => {}
        }
        Ok(Transition::None)
    }

    fn draw(&self, ctx: &mut Context, scr: &mut Screen, theme: &dyn Theme) -> Result<(), SolError> {
        const COUNT_W: u16 = 8;
        const PERCENT_W: u16 = 7;
        let x = ctx.w / 2 - (self.width + 2) / 2;
        let y = ctx.h / 2 - (self.height + 2) / 2;

        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        scr.draw_frame(x, y, self.width + 2, self.height + 2, Border::Double);
        let name_w = self.width - 2 * COUNT_W - PERCENT_W - 3;

        let (xpos, ypos) = (x + 1, y + 1);
        let titles: Vec<&'static str> = vec!["Solitaire", "Played", "Won", "%"];
        let widths: Vec<u16> = vec![name_w, COUNT_W, COUNT_W, PERCENT_W];
        let mut shift = 0u16;
        for (title, width) in titles.iter().zip(widths.iter()) {
            let dx = shift_in(title, *width) + shift;
            scr.write_string(title, xpos + dx, ypos);
            shift += width + 1;
        }

        scr.write_char('│', xpos + name_w, ypos);
        scr.write_char('│', xpos + name_w + COUNT_W + 1, ypos);
        scr.write_char('│', xpos + name_w + COUNT_W * 2 + 2, ypos);
        scr.write_hline(xpos, ypos + 1, self.width, Border::Single);
        scr.write_char('┴', xpos + name_w, ypos + 1);
        scr.write_char('┴', xpos + name_w + COUNT_W + 1, ypos + 1);
        scr.write_char('┴', xpos + name_w + COUNT_W * 2 + 2, ypos + 1);

        for idx in 0..self.height - 2 {
            if idx >= self.top + self.sols.len() as u16 {
                break;
            }
            let n = usize::from(self.top + idx);
            let (fg, bg) = if n == self.selected as usize { theme.menu_selected_item() } else { theme.base_colors() };
            scr.colors(fg, bg);
            scr.write_hline(x + 1, y + 3 + idx, self.width, Border::None);
            scr.write_string(&self.sols[n], x + 1, y + 3 + idx);

            let stats = ctx.stats.game_stat(&self.sols[n]);
            let played_str = if stats.played == 0 {
                String::new()
            } else {
                format!("{:>width$}", stats.played, width = COUNT_W as usize)
            };
            let won_str =
                if stats.won == 0 { String::new() } else { format!("{:>width$}", stats.won, width = COUNT_W as usize) };
            let prc_str = if stats.played == 0 {
                String::new()
            } else if stats.won == 0 {
                "0.0".to_string()
            } else if stats.played == stats.won {
                "100.0".to_string()
            } else {
                format!("{:5.1}", stats.won as f32 / stats.played as f32)
            };
            let percent_str = format!("{:>width$}", prc_str, width = PERCENT_W as usize);
            scr.write_string(&played_str, x + 1 + name_w + 1, y + 3 + idx);
            scr.write_string(&won_str, x + 1 + name_w + COUNT_W + 2, y + 3 + idx);
            scr.write_string(&percent_str, x + 1 + name_w + COUNT_W * 2 + 2, y + 3 + idx);
        }
        Ok(())
    }
}
