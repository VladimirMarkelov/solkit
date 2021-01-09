use std::time::{Duration, SystemTime};

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
    pub(crate) elapsed: Duration,
    started: SystemTime,
}

pub(crate) trait Strategy {
    fn process_event(&mut self, ctx: &mut Context, scr: &mut Screen, event: Event) -> Result<Transition, SolError>;
    fn draw(&self, ctx: &mut Context, scr: &mut Screen, theme: &dyn Theme) -> Result<(), SolError>;
    fn on_activate(&self, ctx: &mut Context);
    fn on_deactivate(&self, ctx: &mut Context);
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
            elapsed: Duration::new(0, 0),
            started: SystemTime::now(),
        }
    }
    pub(crate) fn pause(&mut self) {
        if let Ok(elapsed) = self.started.elapsed() {
            self.elapsed += elapsed;
        }
    }
    pub(crate) fn unpause(&mut self) {
        self.started = SystemTime::now();
    }
    pub(crate) fn reset(&mut self) {
        self.started = SystemTime::now();
        self.elapsed = Duration::new(0, 0);
    }
}
