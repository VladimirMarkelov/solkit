use std::env;
use std::process::exit;

use getopts::{Matches, Options};

const APP_NAME: &str = "Solitaire Kit";

// Options passed via commnd-line
pub(crate) struct CliOpts {
    pub(crate) dark: bool,
    pub(crate) filename: String,
    pub(crate) logging: bool,
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options] [path-to-custom-solitaire-rules]", program);
    print!("{}", opts.usage(&brief));
}

pub(crate) fn parse_args() -> CliOpts {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut cli = CliOpts { dark: true, filename: String::new(), logging: false };

    let mut opts = Options::new();
    opts.optflag("h", "help", "Show this help");
    opts.optopt("t", "theme", "Choose UI theme", "dark | classic");
    opts.optflag("v", "version", "Show application version");
    opts.optflag("", "log", "Enable logging");

    let matches: Matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            print_usage(&program, &opts);
            exit(0);
        }
    };

    if matches.opt_present("version") {
        let version = env!("CARGO_PKG_VERSION");
        println!("{} Version {}", APP_NAME, version);
        exit(0);
    }

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        exit(0);
    }
    cli.logging = matches.opt_present("log");

    if let Some(val) = matches.opt_str("t") {
        cli.dark = match val.to_lowercase().as_str() {
            "dark" => true,
            "classic" => false,
            _ => {
                print_usage(&program, &opts);
                exit(0);
            }
        };
    }

    if !matches.free.is_empty() {
        cli.filename = matches.free[0].to_string();
    }

    cli
}
