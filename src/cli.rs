#![allow(clippy::default_trait_access)]

use std::convert::Infallible;
use std::path::PathBuf;
use std::str::FromStr;

use tracing_subscriber::filter::LevelFilter;

const DEFAULT_VERBOSITY: isize = if cfg!(debug_assertions) { 4 } else { 3 };
const VERBOSITY_LEVELS: &[LevelFilter] = &[
    LevelFilter::OFF,
    LevelFilter::ERROR,
    LevelFilter::WARN,
    LevelFilter::INFO,
    LevelFilter::DEBUG,
    LevelFilter::TRACE,
];
#[allow(clippy::cast_possible_wrap)]
const MAX_VERBOSITY: isize = VERBOSITY_LEVELS.len() as isize - 1;

#[allow(clippy::print_stderr)]
pub(crate) fn print_help(bin: &str) {
    eprintln!(
        concat!(
            env!("BBL2CSV_VERSION"),
            "\n",
            env!("CARGO_PKG_DESCRIPTION"),
            "

USAGE: {bin} [options] <log>...

OPTIONS:
  -i, --index <index>             Choose which log(s) should be decoded or omit to decode all
                                  (applies to all files & can be repeated)
      --limits                    Print the limits and range of each field (TODO)
      --altitude-offset <offset>  Set the altitude offset in meters (TODO)
      --gps                       Write GPS data into .gps.csv files
  -f, --filter <fields>           Select fields to output by name, excluding any suffixed index
                                  (comma separated)
  -F, --gps-filter <fields>       Select gps fields to output by name, like --filter. Implies --gps
  -v, --verbose                   Increase debug output
  -q, --quiet                     Reduce output up to {max_quiet} times
      --color <when>              Set when to enable color [auto, always, never]
  -h, --help                      Print this help
  -V, --version                   Print version information"
        ),
        bin = bin,
        max_quiet = DEFAULT_VERBOSITY,
    );
}

#[allow(clippy::print_stderr)]
pub(crate) fn print_version() {
    eprintln!(env!("BBL2CSV_VERSION"));
}

pub(crate) enum Action {
    Run(Cli),
    Help,
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Color {
    #[default]
    Auto,
    Always,
    Never,
}

impl FromStr for Color {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Color::Auto),
            "always" => Ok(Color::Always),
            "never" => Ok(Color::Never),
            _ => Err("invalid value for --color"),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(unused, clippy::default_trait_access)]
pub(crate) struct Cli {
    pub index: Vec<usize>,
    pub limits: bool,
    pub altitude_offset: i16,
    pub gps: bool,
    pub filter: Option<Vec<String>>,
    pub gps_filter: Option<Vec<String>>,
    pub verbosity: LevelFilter,
    pub color: Color,
    pub logs: Vec<PathBuf>,
}

impl Cli {
    pub(crate) fn parse(mut parser: lexopt::Parser) -> Result<Action, lexopt::Error> {
        use lexopt::prelude::*;

        fn parse_filter(parser: &mut lexopt::Parser) -> Result<Vec<String>, lexopt::Error> {
            parser.value()?.parse_with::<_, _, Infallible>(|s| {
                Ok(s.split(',')
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty())
                    .collect())
            })
        }

        let mut index = Vec::new();
        let mut limits = false;
        let mut altitude_offset = 0;
        let mut gps = false;
        let mut filter = None;
        let mut gps_filter = None;
        let mut verbosity = DEFAULT_VERBOSITY;
        let mut color = Color::Auto;
        let mut logs = Vec::new();

        while let Some(arg) = parser.next()? {
            match arg {
                Short('i') | Long("index") => index.push(parser.value()?.parse()?),
                Long("limits") => limits = true,
                Long("altitude-offset") => altitude_offset = parser.value()?.parse()?,
                Long("gps") => gps = true,
                Short('f') | Long("filter") => {
                    filter = Some(parse_filter(&mut parser)?);
                }
                Short('F') | Long("gps-filter") => {
                    gps = true;
                    gps_filter = Some(parse_filter(&mut parser)?);
                }
                Short('v') | Long("verbose") => verbosity += 1,
                Short('q') | Long("quiet") => verbosity -= 1,
                Long("color") => color = parser.value()?.parse()?,
                Short('h') | Long("help") => return Ok(Action::Help),
                Short('V') | Long("version") => return Ok(Action::Version),
                Value(value) => logs.push(value.into()),

                Short(_) | Long(_) => return Err(arg.unexpected()),
            }
        }

        Ok(Action::Run(Cli {
            index,
            limits,
            altitude_offset,
            gps,
            filter,
            gps_filter,
            verbosity: verbosity_from_int(verbosity),
            color,
            logs,
        }))
    }

    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        if self.logs.is_empty() {
            return Err("at least one log file is required");
        }

        Ok(())
    }

    pub(crate) fn enable_color<S: is_terminal::IsTerminal>(&self, stream: S) -> bool {
        match self.color {
            Color::Auto => stream.is_terminal(),
            Color::Always => true,
            Color::Never => false,
        }
    }
}

fn verbosity_from_int(verbosity: isize) -> LevelFilter {
    let index = verbosity.clamp(0, MAX_VERBOSITY).unsigned_abs();
    VERBOSITY_LEVELS[index]
}
