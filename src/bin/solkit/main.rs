#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

mod buffer;
mod choose_stg;
mod config;
mod final_stg;
mod gstate;
mod help_stg;
mod loader;
mod opts;
mod play_stg;
mod primitive;
mod rules;
mod stats;
mod strategy;
mod theme;
mod ui;
mod userconf;

use std::fs::File;
use std::io::{stdin, stdout, Write};

use anyhow::{anyhow, Result};
use crossterm::event::{read, EnableMouseCapture};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode, ClearType};
use crossterm::tty::IsTty;
use crossterm::{
    execute, queue,
    style::{self, Color},
};
use simplelog::*;

use choose_stg::ChooseStg;
use final_stg::FinalStg;
use help_stg::HelpStg;
use play_stg::PlayStg;
use primitive::Screen;
use strategy::{Context, Strategy, Transition, TransitionStage};

fn scr_reset(scr: &mut Screen) {
    scr.kind(0);
    scr.clear();
    scr.colors(Color::White, Color::Black);
}

fn main_loop(cli: &opts::CliOpts) -> Result<()> {
    let (cols, rows) = terminal::size()?;
    let mut scr = Screen::new(cols, rows)?;
    let mut stdout = stdout();
    if !stdin().is_tty() {
        return Err(anyhow!("stdin is not TTY"));
    }
    execute!(stdout, EnableMouseCapture)?;

    let filename = if cli.filename.is_empty() { None } else { Some(cli.filename.clone()) };
    let rules = rules::load_rules(filename)?;
    info!("Loaded from {:?} - {}", &cli.filename, rules.len());

    let mut ctx = Context::new(cols, rows);
    let mut user_conf = userconf::UserConf::load();
    if user_conf.last_played.is_empty() {
        let mut sols: Vec<String> = Vec::new();
        for (name, _cfg) in rules.iter() {
            sols.push(name.clone());
        }
        if !sols.is_empty() {
            sols.sort();
            user_conf.last_played = sols[0].clone();
        }
    }
    ctx.name = user_conf.last_played.clone();
    ctx.custom = !cli.filename.is_empty();

    let mut stg: Box<dyn Strategy> = Box::new(ChooseStg::new(&rules, &mut ctx).unwrap()); // TODO:
    let mut stages: Vec<Box<dyn Strategy>> = Vec::new();

    let dark = theme::DarkTheme::new(true);
    let light = theme::LightTheme::new(true);
    let thm: &dyn theme::Theme = if cli.dark { &dark } else { &light };
    let (fg, bg) = thm.base_colors();
    scr.colors(fg, bg);
    scr.clear();
    execute!(stdout, style::SetForegroundColor(fg), style::SetBackgroundColor(bg), terminal::Clear(ClearType::All),)?;

    loop {
        stg.draw(&mut ctx, &mut scr, thm)?;
        scr.flush(&mut stdout)?;
        stdout.flush()?;
        let ev = read()?;
        let trans = stg.process_event(&mut ctx, &mut scr, ev)?;
        match trans {
            Transition::None => {}
            Transition::Pop => match stages.pop() {
                None => return Ok(()),
                Some(_) => scr_reset(&mut scr),
            },
            Transition::Exit => {
                if ctx.moved {
                    ctx.stats.update_stat(&ctx.name, ctx.won);
                    if !ctx.custom {
                        ctx.stats.save();
                    }
                }
                stages.clear();
                user_conf.last_played = ctx.name.clone();
                if !ctx.custom {
                    user_conf.save();
                }
                return Ok(());
            }
            Transition::Push(st) => {
                stages.push(stg);
                scr_reset(&mut scr);
                stg = match st {
                    TransitionStage::EndDialog => Box::new(FinalStg::new(&mut ctx).unwrap()),
                    TransitionStage::Play => {
                        ctx.state.clear_mark();
                        ctx.state.clear_hints();
                        Box::new(PlayStg::new(&rules, &mut ctx).unwrap()) // TODO:
                    }
                    TransitionStage::Choose => {
                        ctx.state.clear_mark();
                        ctx.state.clear_hints();
                        Box::new(ChooseStg::new(&rules, &mut ctx).unwrap()) // TODO:
                    }
                    TransitionStage::HelpDialog => Box::new(HelpStg::new(&mut ctx).unwrap()),
                };
            }
            Transition::Replace(st) => {
                stages.clear();
                if ctx.moved {
                    ctx.stats.update_stat(&ctx.name, ctx.won);
                    if !ctx.custom {
                        ctx.stats.save();
                    }
                }
                ctx.moved = false;
                ctx.won = false;
                scr_reset(&mut scr);
                stg = match st {
                    TransitionStage::EndDialog => Box::new(FinalStg::new(&mut ctx).unwrap()),
                    TransitionStage::Play => {
                        ctx.state.clear_mark();
                        ctx.state.clear_hints();
                        Box::new(PlayStg::new(&rules, &mut ctx).unwrap()) // TODO:
                    }
                    TransitionStage::Choose => {
                        ctx.state.clear_mark();
                        ctx.state.clear_hints();
                        Box::new(ChooseStg::new(&rules, &mut ctx).unwrap()) // TODO:
                    }
                    _ => panic!("unimplemented"),
                };
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = opts::parse_args();

    if cli.logging {
        let cb = ConfigBuilder::new().set_time_format("[%Y-%m-%d %H:%M:%S%.3f]".to_string()).build();
        CombinedLogger::init(vec![WriteLogger::new(LevelFilter::Info, cb, File::create("app.log").unwrap())]).unwrap();
    }

    let mut stdout = stdout();
    // TODO: cursor::Hide,
    execute!(stdout, terminal::EnterAlternateScreen)?;
    enable_raw_mode()?;

    let err = main_loop(&cli);

    // TODO: cursor::Show,
    queue!(stdout, style::ResetColor, terminal::LeaveAlternateScreen)?;
    stdout.flush()?;

    disable_raw_mode()?;
    if err.is_err() {
        err
    } else {
        Ok(())
    }
}
