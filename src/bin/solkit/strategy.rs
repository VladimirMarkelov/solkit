use crossterm::event::Event;

use solkit::err::SolError;

use crate::gstate::GameState;
use crate::primitive::Screen;
use crate::stats::Stats;
use crate::theme::Theme;

pub(crate) enum TransitionStage {
    Play,
    Choose,
    EndDialog,
    HelpDialog,
}

pub(crate) enum Transition {
    None,
    Pop,
    Exit,
    Push(TransitionStage),
    Replace(TransitionStage),
}

pub(crate) struct Context {
    pub(crate) name: String,
    pub(crate) state: GameState,
    pub(crate) w: u16, // screen width
    pub(crate) h: u16, // screen height
    pub(crate) stats: Stats,
    pub(crate) moved: bool, // to avoid changing stats if no move was done
    pub(crate) won: bool,
    pub(crate) custom: bool, // app launched with a custom solitaire
}

pub(crate) trait Strategy {
    fn process_event(&mut self, ctx: &mut Context, scr: &mut Screen, event: Event) -> Result<Transition, SolError>;
    fn draw(&self, ctx: &mut Context, scr: &mut Screen, theme: &dyn Theme) -> Result<(), SolError>;
}

impl Context {
    pub(crate) fn new(cols: u16, rows: u16) -> Self {
        Context {
            name: String::new(),
            state: GameState::new(),
            w: cols,
            h: rows,
            stats: Stats::load(),
            moved: false,
            won: false,
            custom: false,
        }
    }
}
