use crossterm::event::{Event, KeyCode};
use crossterm::terminal;
use unicode_width::UnicodeWidthStr;

use solkit::err::SolError;

use crate::primitive::{Border, Screen};
use crate::strategy::{Context, Strategy, Transition, TransitionStage};
use crate::theme::Theme;

const ITEM_COUNT: usize = 4;
const ITEM_HEIGHT: u16 = 3;
const MENU_WIDTH: u16 = 28;
const MENU_ITEMS: [&str; 4] = ["Back to game", "Play again", "Choose solitaire", "Exit application"];

// menu displayed on ESC or when a solitaire is solved
pub(crate) struct FinalStg {
    selected: usize,
}

impl FinalStg {
    pub(crate) fn new(_ctx: &mut Context) -> Result<Self, SolError> {
        Ok(FinalStg { selected: 0 })
    }
}

impl Strategy for FinalStg {
    fn process_event(&mut self, ctx: &mut Context, scr: &mut Screen, event: Event) -> Result<Transition, SolError> {
        match event {
            Event::Key(ev) => match ev.code {
                KeyCode::Esc => return Ok(Transition::Pop),
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected == 0 {
                        self.selected = ITEM_COUNT - 1;
                    } else {
                        self.selected -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected == ITEM_COUNT - 1 {
                        self.selected = 0;
                    } else {
                        self.selected += 1;
                    }
                }

                KeyCode::Enter => {
                    if self.selected == ITEM_COUNT - 1 {
                        return Ok(Transition::Exit);
                    } else if self.selected == 0 {
                        return Ok(Transition::Pop);
                    } else if self.selected == 1 {
                        return Ok(Transition::Replace(TransitionStage::Play));
                    } else if self.selected == 2 {
                        return Ok(Transition::Replace(TransitionStage::Choose));
                    }
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
            }
            _ => {}
        }
        Ok(Transition::None)
    }

    fn draw(&self, ctx: &mut Context, scr: &mut Screen, theme: &dyn Theme) -> Result<(), SolError> {
        let x = ctx.w / 2 - MENU_WIDTH / 2;
        let h = ITEM_COUNT as u16 * ITEM_HEIGHT + 2;
        let y = ctx.h / 2 - h / 2;

        let (fg, bg) = theme.base_colors();
        scr.colors(fg, bg);
        scr.draw_frame(x, y, MENU_WIDTH, h, Border::Double);
        for (idx, item) in MENU_ITEMS.iter().enumerate() {
            let (fg, bg) = if idx == self.selected { theme.menu_selected_item() } else { theme.base_colors() };
            scr.kind(idx as u16 + 1);
            scr.colors(fg, bg);
            scr.fill_rect(x + 1, y + 1 + idx as u16 * ITEM_HEIGHT, MENU_WIDTH - 2, ITEM_HEIGHT, ' ');
            let slen = item.width();
            let mut shift = MENU_WIDTH / 2 - slen as u16 / 2;
            if slen % 2 == 1 {
                shift -= 1;
            }
            scr.write_string(MENU_ITEMS[idx], x + shift, y + 2 + idx as u16 * ITEM_HEIGHT);
        }
        Ok(())
    }

    fn on_activate(&self, _ctx: &mut Context) {}
    fn on_deactivate(&self, _ctx: &mut Context) {}
}
