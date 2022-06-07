use anyhow::Result;
use clap::Parser;
use fan_controller::{fan_loop, logger::Logger, pwm::Channel};
use log::LevelFilter;
use std::time::Duration;

static LOGGER: Logger = Logger;
static LOG_LEVELS: &[&str; 6] = &["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"];

const DEFAULT_CHANNEL: usize = 0;
const DEFAULT_FREQUENCY: f64 = 25_000.0; // 25kHz for Noctua fans
const DEFAULT_LOG_LEVEL: &str = "DEBUG";
const DEFAULT_SLEEP: u64 = 1;
const INITIAL_DUTY_CYCLE: f64 = 1.0;
const TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";

#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    /// The hardware PWM channel to control. Should be 0 or 1.
    #[clap(short, long, validator = hardware_pwm_channel, default_value_t = DEFAULT_CHANNEL)]
    channel: usize,

    /// PWM frequency to use, in Hz.
    #[clap(short, long, default_value_t = DEFAULT_FREQUENCY)]
    frequency: f64,

    /// The initial duty cycle to start at, from 0.0 to 1.0
    #[clap(short, long, validator = duty_cycle_range, default_value_t = INITIAL_DUTY_CYCLE)]
    duty_cycle: f64,

    /// The path to read the temperature from
    #[clap(short, long, default_value_t = TEMP_PATH.to_string())]
    temp_path: String,

    /// Time, in seconds, to sleep between checking temperature
    #[clap(short, long, default_value_t = DEFAULT_SLEEP)]
    sleep: u64,

    /// Log level
    #[clap(short, long, validator = is_log_level, default_value_t = DEFAULT_LOG_LEVEL.to_string())]
    log_level: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = match args.log_level.as_str() {
        "TRACE" => LevelFilter::Trace,
        "DEBUG" => LevelFilter::Debug,
        "INFO" => LevelFilter::Info,
        "WARN" => LevelFilter::Warn,
        "ERROR" => LevelFilter::Error,
        "OFF" => LevelFilter::Off,
        _ => LevelFilter::Warn,
    };

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log_level))
        .unwrap();

    let channel = match args.channel {
        1 => Channel::Pwm1,
        _ => Channel::Pwm0,
    };

    let sleep = Duration::from_secs(args.sleep);

    fan_loop(
        channel,
        args.frequency,
        args.duty_cycle,
        args.temp_path,
        sleep,
    )?;

    Ok(())
}

fn is_log_level(val: &str) -> Result<(), String> {
    if LOG_LEVELS.iter().any(|&l| l == val) {
        Ok(())
    } else {
        Err(
            "Unknown log level.\nShould be one of: TRACE, DEBUG, INFO, WARN, ERROR, OFF"
                .to_string(),
        )
    }
}

fn hardware_pwm_channel(val: &str) -> Result<(), String> {
    match val {
        "0" | "1" => Ok(()),
        _ => Err("Value has to be either 0 or 1".to_string()),
    }
}

fn duty_cycle_range(val: &str) -> Result<(), String> {
    match val.parse::<f64>() {
        Ok(duty_cycle) => {
            if duty_cycle < 0.0 {
                Err("Value can't be below 0.0".to_string())
            } else if duty_cycle > 1.0 {
                Err("Value can't be above 1.0".to_string())
            } else {
                Ok(())
            }
        }
        Err(_) => Err("Unable to parse duty cycle".to_string()),
    }
}
