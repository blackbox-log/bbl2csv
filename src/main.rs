mod cli;

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::{iter, process};

use blackbox_log::data::ParserEvent;
use blackbox_log::prelude::*;
use blackbox_log::units::{si, Time};
use blackbox_log::{frame, Event, Filter, FilterSet, Value};
use mimalloc::MiMalloc;
use rayon::prelude::*;

use self::cli::{Action, Cli};

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

fn main() {
    // Enable ANSI escapes on Windows
    #[cfg(windows)]
    output_vt100::init();

    let parser = lexopt::Parser::from_env();
    let bin = parser
        .bin_name()
        .unwrap_or(env!("CARGO_BIN_NAME"))
        .to_owned();

    let cli = match Cli::parse(parser) {
        Ok(Action::Run(cli)) => cli,
        Ok(Action::Help) => {
            cli::print_help(&bin);
            process::exit(exitcode::OK)
        }
        Ok(Action::Version) => {
            cli::print_version();
            process::exit(exitcode::OK)
        }
        #[allow(clippy::print_stderr)]
        Err(err) => {
            eprintln!("{err}");
            process::exit(exitcode::USAGE)
        }
    };

    let log_stream = io::stderr;
    tracing_subscriber::fmt()
        .with_max_level(cli.verbosity)
        .with_ansi(cli.enable_color(log_stream()))
        .with_writer(log_stream)
        .init();

    if let Err(err) = cli.validate() {
        tracing::error!("{err}");
        process::exit(exitcode::USAGE);
    }

    let make_filter = |f: Option<Vec<_>>| {
        f.map(|fields| Filter::OnlyFields(fields.iter().collect()))
            .unwrap_or_default()
    };
    let filter = make_filter(cli.filter);
    let filters = FilterSet {
        main: filter.clone(),
        slow: filter,
        gps: make_filter(cli.gps_filter),
    };

    let result = cli.logs.par_iter().try_for_each(|filename| {
        let span = tracing::info_span!("file", name = ?filename);
        let _span = span.enter();

        let data = fs::read(filename).map_err(|error| {
            tracing::error!(%error, "failed to read log file");
            exitcode::IOERR
        })?;

        let file = blackbox_log::File::new(&data);

        file.iter()
            .enumerate()
            .par_bridge()
            .try_for_each(|(i, headers)| {
                let human_i = i + 1;

                let span = tracing::info_span!("log", index = human_i);
                let _span = span.enter();

                let headers = headers.map_err(|err| {
                    tracing::debug!("header parse error: {err}");
                    exitcode::DATAERR
                })?;

                let mut parser = headers.data_parser_with_filters(&filters);

                let main_frame_def = parser.main_frame_def();
                let slow_frame_def = parser.slow_frame_def();
                let field_names = main_frame_def
                    .iter()
                    .map(|f| f.name)
                    .chain(slow_frame_def.iter().map(|f| f.name));
                let field_names = iter::once("time").chain(field_names);

                let mut out = get_output(filename, human_i, "csv")?;
                if let Err(error) = write_csv_line(&mut out, field_names) {
                    tracing::error!(%error, "failed to write csv header");
                    return Err(exitcode::IOERR);
                }

                let mut gps_out = match parser.gps_frame_def() {
                    Some(def) if cli.gps => {
                        let mut out = get_output(filename, human_i, "gps.csv")?;

                        if let Err(error) = write_csv_line(&mut out, def.iter().map(|f| f.name)) {
                            tracing::error!(%error, "failed to write gps csv header");
                            return Err(exitcode::IOERR);
                        }

                        Some(out)
                    }
                    _ => None,
                };

                let mut events_out = get_output(filename, human_i, "events.csv")?;
                if let Err(error) = writeln!(events_out, "time,event") {
                    tracing::error!(%error, "failed to write events csv header");
                    return Err(exitcode::IOERR);
                }

                let mut slow: String = ",".repeat(parser.slow_frame_def().len().saturating_sub(1));
                let mut last_time = 0;
                while let Some(frame) = parser.next() {
                    match frame {
                        ParserEvent::Event(event) => {
                            if let Err(error) = write_event(&mut events_out, event, last_time) {
                                tracing::error!(%error, "failed to write event");
                                return Err(exitcode::IOERR);
                            }
                        }
                        ParserEvent::Slow(frame) => {
                            slow.clear();
                            format_slow_frame(&mut slow, frame);
                        }
                        ParserEvent::Main(main) => {
                            last_time = main.time_raw();
                            if let Err(error) = write_main_frame(&mut out, main, &slow) {
                                tracing::error!(%error, "failed to write csv");
                                return Err(exitcode::IOERR);
                            }
                        }
                        ParserEvent::Gps(gps) => {
                            last_time = gps.time_raw();
                            if let Some(ref mut out) = gps_out {
                                if let Err(error) = write_gps_frame(out, gps) {
                                    tracing::error!(%error, "failed to write gps csv");
                                    return Err(exitcode::IOERR);
                                }
                            }
                        }
                    }
                }

                if let Err(error) = out.flush() {
                    tracing::error!(%error, "failed to flush csv");
                    return Err(exitcode::IOERR);
                }

                if let Some(Err(error)) = gps_out.map(|mut out| out.flush()) {
                    tracing::error!(%error, "failed to flush gps csv");
                    return Err(exitcode::IOERR);
                }

                Ok(())
            })
    });

    if let Err(code) = result {
        process::exit(code);
    }
}

fn get_output(
    filename: &Path,
    index: usize,
    extension: &str,
) -> Result<BufWriter<File>, exitcode::ExitCode> {
    let mut out = filename.to_owned();
    out.set_extension(format!("{index:0>2}.{extension}"));

    let file = File::create(&out).map_err(|error| {
        tracing::error!(%error, file = %out.display(), "failed to open output file");
        exitcode::CANTCREAT
    })?;

    tracing::info!("Writing to '{}'", out.display());

    Ok(BufWriter::new(file))
}

fn write_main_frame(out: &mut impl Write, main: frame::MainFrame, slow: &str) -> io::Result<()> {
    out.write_all(format_time(main.time()).as_bytes())?;

    for field in main.iter().map(|v| format_value(v.into())) {
        out.write_all(b",")?;
        out.write_all(field.as_bytes())?;
    }

    if !slow.is_empty() {
        out.write_all(b",")?;
    }

    out.write_all(slow.as_bytes())?;
    out.write_all(b"\n")
}

fn format_slow_frame(out: &mut String, slow: frame::SlowFrame) {
    let mut fields = slow.iter().map(|v| format_value(v.into()));

    if let Some(first) = fields.next() {
        out.push_str(&first);

        for field in fields {
            out.push(',');
            out.push_str(&field);
        }
    }
}

fn write_gps_frame(out: &mut impl Write, gps: frame::GpsFrame) -> io::Result<()> {
    let time = format_time(gps.time());
    let fields = gps.iter().map(Value::from).map(format_value);

    write_csv_line(out, iter::once(time).chain(fields))
}

fn write_event(out: &mut impl Write, event: Event, last_time: u64) -> io::Result<()> {
    let (name, time) = match event {
        Event::SyncBeep(time) => ("Sync beep", Some(time)),
        Event::InflightAdjustment { .. } => ("Inflight adjustment", None),
        Event::Resume { time, .. } => ("Logging resume", Some(time.into())),
        Event::Disarm(_) => ("Disarm", None),
        Event::FlightMode { .. } => ("Flight mode change", None),
        Event::ImuFailure { .. } => ("IMU failure", None),
        Event::End { .. } => ("End", None),
    };

    writeln!(out, "{},{name}", time.unwrap_or(last_time))
}

fn format_time(time: Time) -> String {
    format!("{:.0}", time.get::<si::time::microsecond>())
}

fn format_value(value: Value) -> String {
    fn format_float(f: f64) -> String {
        format!("{f:.2}")
    }

    match value {
        Value::Amperage(a) => format_float(a.get::<si::electric_current::ampere>()),
        Value::Voltage(v) => format_float(v.get::<si::electric_potential::volt>()),
        Value::Acceleration(a) => {
            format_float(a.get::<si::acceleration::meter_per_second_squared>())
        }
        Value::Rotation(r) => format_float(r.get::<si::angular_velocity::degree_per_second>()),
        Value::FlightMode(f) => f.to_string(),
        Value::State(s) => s.to_string(),
        Value::FailsafePhase(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::GpsCoordinate(c) => format!("{c:.7}"),
        Value::Altitude(a) => format!("{:.0}", a.get::<si::length::meter>()),
        Value::Velocity(v) => format_float(v.get::<si::velocity::meter_per_second>()),
        Value::GpsHeading(h) => format!("{h:.1}"),
        Value::Unsigned(u) => u.to_string(),
        Value::Signed(s) => s.to_string(),
    }
}

fn write_csv_line<T: AsRef<str>>(
    out: &mut impl Write,
    mut fields: impl Iterator<Item = T>,
) -> io::Result<()> {
    if let Some(first) = fields.next() {
        out.write_all(first.as_ref().as_bytes())?;

        for s in fields {
            out.write_all(b",")?;
            out.write_all(s.as_ref().as_bytes())?;
        }
    }

    out.write_all(b"\n")?;

    Ok(())
}
