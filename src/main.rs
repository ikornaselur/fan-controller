use anyhow::Result;
use clap::{Parser, Subcommand};
use fan_controller::{config, fan_loop, logger::Logger, mqtt::MqttConfig, pwm::Channel};
use log::LevelFilter;
use std::time::Duration;

fn gethostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

mod service;
mod update;

static LOGGER: Logger = Logger;

const DEFAULT_CHANNEL: usize = 0;
const DEFAULT_FREQUENCY: f64 = 25_000.0; // 25kHz for Noctua fans
const DEFAULT_LOG_LEVEL: &str = "DEBUG";
const DEFAULT_SLEEP: u64 = 1;
const INITIAL_DUTY_CYCLE: f64 = 1.0;
const TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";

const DEFAULT_TARGET_TEMP: f32 = 45.0;
const DEFAULT_KP: f32 = 0.02;
const DEFAULT_KI: f32 = 0.001;
const DEFAULT_KD: f32 = 0.01;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the fan control loop
    Run(RunArgs),
    /// Install and enable the systemd service
    Install(InstallArgs),
    /// Stop, disable, and remove the systemd service
    Uninstall,
    /// Self-update from the latest GitHub release
    Update,
}

#[derive(Parser, Debug)]
struct RunArgs {
    /// The hardware PWM channel to control (0 or 1)
    #[arg(short, long, default_value_t = DEFAULT_CHANNEL, value_parser = parse_channel)]
    channel: usize,

    /// PWM frequency to use, in Hz
    #[arg(short, long, default_value_t = DEFAULT_FREQUENCY)]
    frequency: f64,

    /// The initial duty cycle to start at (0.0 to 1.0)
    #[arg(short, long, default_value_t = INITIAL_DUTY_CYCLE, value_parser = parse_duty_cycle)]
    duty_cycle: f64,

    /// The path to read the temperature from
    #[arg(short, long, default_value_t = TEMP_PATH.to_string())]
    temp_path: String,

    /// Time, in seconds, to sleep between checking temperature
    #[arg(short, long, default_value_t = DEFAULT_SLEEP)]
    sleep: u64,

    /// Log level (TRACE, DEBUG, INFO, WARN, ERROR, OFF)
    #[arg(short, long, default_value_t = DEFAULT_LOG_LEVEL.to_string(), value_parser = parse_log_level)]
    log_level: String,

    /// Target temperature in degrees C
    #[arg(long)]
    target_temp: Option<f32>,

    /// PID proportional gain
    #[arg(long)]
    kp: Option<f32>,

    /// PID integral gain
    #[arg(long)]
    ki: Option<f32>,

    /// PID derivative gain
    #[arg(long)]
    kd: Option<f32>,

    /// Number of temperature readings to average (smooths sensor noise)
    #[arg(long, default_value_t = 1)]
    temp_samples: usize,

    /// MQTT broker address (enables MQTT when set)
    #[arg(long)]
    mqtt_broker: Option<String>,

    /// MQTT broker port
    #[arg(long, default_value_t = 1883)]
    mqtt_port: u16,

    /// MQTT topic prefix (defaults to fan_controller_{hostname})
    #[arg(long)]
    mqtt_prefix: Option<String>,
}

#[derive(Parser, Debug)]
struct InstallArgs {
    /// Arguments to pass to the 'run' subcommand in the service file.
    /// Example: -- --target-temp 50 --kp 0.03 --mqtt-broker 192.168.1.100
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    run_args: Vec<String>,

    /// MQTT username to bake into the service file (as Environment= line)
    #[arg(long)]
    mqtt_username: Option<String>,

    /// MQTT password to bake into the service file (as Environment= line)
    #[arg(long)]
    mqtt_password: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run(args) => run(args),
        Command::Install(args) => {
            let mqtt_username = args.mqtt_username.or_else(|| std::env::var("MQTT_USERNAME").ok());
            let mqtt_password = args.mqtt_password.or_else(|| std::env::var("MQTT_PASSWORD").ok());
            service::install(&args.run_args, &mqtt_username, &mqtt_password)
        }
        Command::Uninstall => service::uninstall(),
        Command::Update => update::update(),
    }
}

fn run(args: RunArgs) -> Result<()> {
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

    // Load saved config, then layer: CLI arg > config file > default
    let saved = config::load().unwrap_or_else(|e| {
        eprintln!("Warning: failed to load config: {}", e);
        None
    });

    let target_temp = args.target_temp.or(saved.as_ref().map(|c| c.target_temp)).unwrap_or(DEFAULT_TARGET_TEMP);
    let kp = args.kp.or(saved.as_ref().map(|c| c.kp)).unwrap_or(DEFAULT_KP);
    let ki = args.ki.or(saved.as_ref().map(|c| c.ki)).unwrap_or(DEFAULT_KI);
    let kd = args.kd.or(saved.as_ref().map(|c| c.kd)).unwrap_or(DEFAULT_KD);

    // Save resolved config (persists CLI overrides)
    let cfg = config::Config { target_temp, kp, ki, kd };
    if let Err(e) = config::save(&cfg) {
        eprintln!("Warning: failed to save config: {}", e);
    }

    let channel = match args.channel {
        1 => Channel::Pwm1,
        _ => Channel::Pwm0,
    };

    let sleep = Duration::from_secs(args.sleep);

    let mqtt_config = args.mqtt_broker.map(|broker| {
        let hostname = gethostname();
        let prefix = args.mqtt_prefix.unwrap_or_else(|| {
            format!("fan_controller_{}", hostname)
        });
        MqttConfig {
            broker,
            port: args.mqtt_port,
            prefix,
            hostname,
            username: std::env::var("MQTT_USERNAME").ok(),
            password: std::env::var("MQTT_PASSWORD").ok(),
        }
    });

    fan_loop(
        channel,
        args.frequency,
        args.duty_cycle,
        args.temp_path,
        sleep,
        target_temp,
        kp,
        ki,
        kd,
        mqtt_config,
        args.temp_samples,
    )
}

fn parse_channel(s: &str) -> Result<usize, String> {
    match s {
        "0" | "1" => Ok(s.parse().unwrap()),
        _ => Err("channel must be 0 or 1".to_string()),
    }
}

fn parse_duty_cycle(s: &str) -> Result<f64, String> {
    let val: f64 = s.parse().map_err(|_| "unable to parse duty cycle")?;
    if !(0.0..=1.0).contains(&val) {
        return Err("duty cycle must be between 0.0 and 1.0".to_string());
    }
    Ok(val)
}

fn parse_log_level(s: &str) -> Result<String, String> {
    match s {
        "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" | "OFF" => Ok(s.to_string()),
        _ => Err("must be one of: TRACE, DEBUG, INFO, WARN, ERROR, OFF".to_string()),
    }
}
