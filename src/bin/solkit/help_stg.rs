use crossterm::event::{Event, KeyCode};
use crossterm::terminal;

use solkit::err::SolError;

use crate::primitive::{Border, Screen};
use crate::strategy::{Context, Strategy, Transition};
use crate::theme::Theme;

const DLG_WIDTH: u16 = 66;
const DLG_HEIGHT: u16 = 22;
const DLG_ITEMS: [&str; 20] = [
    "`Ctrl`+`q` - immediate exit",
    "`Left` or `h` - select the previous pile",
    "`Right` or `l` - select the next pile",
    "`Up` or `k` - select an upper card in a pile",
    "`Down` or `j` - select a lower card in a pile",
    "`1`-`0` - select a pile of play area. If the pile ",
    "   is already selected, the key acts as `Up`",
    "`Shift`+`1` - `Shift`+`0` (`!` - `(`)  - select a pile of play area.",
    "   If the pile is already selected, the key acts as `Down`",
    "`d`, `f`, `c` - select first pile in a group if the group exists.",
    "   If any pile in the group is already selected, the selection ",
    "   moves to the next pile in the group.",
    "`s` - `s`how hints: highlight cards that can be played",
    "`u` - `u`ndo last move",
    "`shift`+`r`(`R`) or `F5` - `r`edeal: start a new game",
    "`Space` - mark/unmark a card for the next move",
    "`Enter` or `m` - move a marked card to the currently selected one.",
    "    If the currently selected card is a marked one or no card is",
    "    marked, the currently selected card moves to the first valid",
    "    location (priority: foundation, play area, additional area).",
];

pub(crate) struct HelpStg {}

impl HelpStg {
    pub(crate) fn new(_ctx: &mut Context) -> Result<Self, SolError> {
        Ok(HelpStg {})
    }
}

impl Strategy for HelpStg {
    fn process_event(&mut self, ctx: &mut Context, scr: &mut Screen, event: Event) -> Result<Transition, SolError> {
        match event {
            Event::Key(ev) => {
                if let KeyCode::Esc = ev.code {
                    return Ok(Transition::Pop);
                }
            }
            Event::Resize(_, _) => {
                let (width, height) = terminal::size().unwrap(); //TODO:
                if width < 60 || height < 25 {
                    eprintln!("Requires terminal width at least 60 and height at least 25 characters");
                    return Ok(Transition::Exit);
                }
                if let Err(e) = scr.resize(width, height) {
                    eprintln!("Failed to resize: {:?}", e);
                    return Ok(Transition::Exit);
                }
                ctx.w = width;
                ctx.h = height;
            }
            _ => {
                // TODO: mouse events
            }
        }
        Ok(Transition::None)
    }

    fn draw(&self, ctx: &mut Context, scr: &mut Screen, theme: &dyn Theme) -> Result<(), SolError> {
        let x = ctx.w / 2 - DLG_WIDTH / 2;
        let y = ctx.h / 2 - DLG_HEIGHT / 2;

        let (fg, bg) = theme.base_colors();
        let (wfg, _wbg) = theme.win_msg();

        scr.colors(fg, bg);
        scr.draw_frame(x, y, DLG_WIDTH, DLG_HEIGHT, Border::Double);
        scr.write_string(" Hotkeys ", x + 1, y);

        for (idx, item) in DLG_ITEMS.iter().enumerate() {
            scr.write_string_highlight(item, x + 1, y + 1 + idx as u16, wfg);
        }
        Ok(())
    }
}
