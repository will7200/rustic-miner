#[macro_use]
extern crate serde_derive;
extern crate serde;

mod common;
mod dero;

use std::ffi::OsString;
use std::path::PathBuf;
use std::{process, thread};
use std::any::Any;
use std::borrow::Borrow;
use std::cmp::min;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use ctrlc;
use clap::{ErrorKind, Parser, Subcommand, CommandFactory, FromArgMatches, Error as ClapError, Command};
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{bounded, Receiver, select, tick};
use crossbeam::sync::ShardedLock;
use fern::colors::{Color, ColoredLevelConfig};
use log::{error, info, trace, LevelFilter, debug, warn, log};
use derohe::rpc::daemon_rpc;
use derohe::rpc::daemon_rpc::GetBlockTemplateResult;
use crate::dero::{Job, Miner, MinerError, WorkGatherer};

/// A fictional versioning CLI
#[derive(Parser)]
#[clap(name = "dero-miner")]
#[clap(about = "A dero miner", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command1: Commands,
    #[clap(short, long, parse(from_occurrences), global = true)]
    verbose: usize,
    #[clap(short, long, global = true, validator = dero::address::validate)]
    /// This address is rewarded when a block is mined successfully.
    wallet_address: Option<String>,
    #[clap(short, long, global = true, default_value_t = num_cpus::get())]
    mining_threads: usize,
    #[clap(short, long, global = true, default_value_t = String::from("127.0.0.1:10100"))]
    /// Miner will connect to daemon RPC on this port.
    daemon_rpc_address: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Mine Dero!
    Mine {},
    /// Run benchmark mode.
    Benchmark {},
    /// Show version.
    Version {},
}

fn format_error<I: CommandFactory>(err: ClapError) -> ClapError {
    let mut cmd = I::command();
    err.format(&mut cmd)
}

fn main() {
    let cli = <Cli as Parser>::parse();
    setup_logger(cli.verbose);
    match &cli.command1 {
        Commands::Mine {} => {
            info!("Running dero miner");
            info!("DERO Stargate HE AstroBWT miner : It is an testing version, use it for testing/evaluations purpose only.");
            if cli.wallet_address.is_none() {
                let mut cmd = <Cli as CommandFactory>::command();
                cmd.error(
                    ErrorKind::EmptyValue,
                    "wallet address must be set",
                ).exit();
            }
            info!("Setting wallet address to {}", cli.wallet_address.as_ref().unwrap());
            if cli.mining_threads > num_cpus::get() {
                warn!("Mining threads is more than available CPUs. This is NOT optimal")
            } else if cli.mining_threads < 1 {
                let mut cmd = <Cli as CommandFactory>::command();
                cmd.error(
                    ErrorKind::ValueValidation,
                    "mining threads is too few",
                ).exit();
            }
            info!("System will use {} thread(s) to mine.", cli.mining_threads);
            start_miner(cli).unwrap()
        }
        Commands::Benchmark {} => {
            info!("Benchmarking")
        }
        Commands::Version {} => {
            println!("{}", common::definitions::VERSION)
        }
    }
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(10);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn start_miner(cli: Cli) -> Result<(), Box<dyn Error>> {
    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(10));
    let wallet_address = cli.wallet_address.as_ref().unwrap().clone();

    let counter = Arc::new(AtomicCell::new(0 as i64));
    let job = Arc::new(RwLock::new(Job {
        ijob: daemon_rpc::GetBlockTemplateResult {
            JobID: "".to_string(),
            Blocktemplate_blob: "".to_string(),
            Blockhashing_blob: "".to_string(),
            Difficulty: "".to_string(),
            Difficultyuint64: 0,
            Height: 0,
            Prev_Hash: "".to_string(),
            EpochMilli: 0,
            Blocks: 0,
            MiniBlocks: 0,
            LastError: "".to_string(),
            Status: "".to_string(),
        },
        count: 0,
    }));
    let wg = WorkGatherer::new(wallet_address, cli.daemon_rpc_address, job.clone());
    let wg_receiver = wg.receiver();
    let jh = thread::spawn(move || wg.get_work());
    for i in 1..=cli.mining_threads.into() {
        let miner = Miner::new(i, job.clone(), counter.clone());
        debug!("Starting miner {}", i);
        thread::spawn(|| {
            miner.start()
        });
    }
    let mut last_counter = 0;
    let mut last_counter_time = SystemTime::now();
    loop {
        select! {
            recv(ticks) -> _ => {
                let icounter = counter.load();
                let time_elasped = SystemTime::now().duration_since(last_counter_time).unwrap();
                info!("{}, {:?}", icounter - last_counter, time_elasped);
                let mining_speed = ((icounter - last_counter) as f64) / ((time_elasped.as_nanos() as f64) / 1000000000.0);
                last_counter = icounter;
                last_counter_time = SystemTime::now();
                let mining_speed_string: String = match mining_speed {
                    _ if mining_speed > 1000000.0 => format!("{} MH/s", mining_speed / 1000000.0),
                    _ if mining_speed > 1000.0 => format!("{} KH/s", mining_speed / 1000.0),
                    _ => format!("{} H/s", mining_speed)
                };

                info!("Mining speed: {}", mining_speed_string);
            }
            recv(wg_receiver) -> val => {
                match val.unwrap() {
                    MinerError::WebSocketError(val) => error!("{}",val)
                }
            }
            recv(ctrl_c_events) -> _ => {
                info!("Goodbye!");
                break;
            }
        }
    }
    Ok(())
}

fn setup_logger(x: usize) -> Result<(), fern::InitError> {
    let level_filter = match x + 1 {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => LevelFilter::Trace,
    };
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::White)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);
    // configure colors for the name of the level.
    // since almost all of them are the same as the color for the whole line, we
    // just clone `colors_line` and overwrite our changes
    let colors_level = colors_line.clone().info(Color::Green);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}{date} {target} {level}{color_line} {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message
            ))
        })
        .level(level_filter)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}