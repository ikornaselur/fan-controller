use cycle::get_duty_cycle;
use log::{debug, info, LevelFilter};
use logger::Logger;
use rppal::pwm::{Channel, Polarity, Pwm};
use std::fs::read_to_string;
use std::{error::Error, thread, time::Duration};

mod cycle;
mod logger;

static LOGGER: Logger = Logger;

const SLEEP: Duration = Duration::from_secs(1);
const FREQUENCY: f64 = 25_000.0; // 25kHz for Noctua fans
const TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";

/// Returns the current temperature in celcius
fn get_temp() -> Result<f32, Box<dyn Error>> {
    let contents: String = read_to_string(TEMP_PATH)?;
    let millicelcius: f32 = contents.parse()?;

    debug!("Temperature read in millicelcius: {}", millicelcius);

    Ok(millicelcius / 1000.0)
}

fn main() -> Result<(), Box<dyn Error>> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
    info!("Initialising..");
    let pwm = Pwm::with_frequency(Channel::Pwm0, FREQUENCY, 1.0, Polarity::Normal, true)?;
    let mut cycle_idx: usize = 0;
    loop {
        let temp = get_temp()?;
        let (new_cycle_idx, duty_cycle) = get_duty_cycle(cycle_idx, temp)?;
        cycle_idx = new_cycle_idx;

        debug!("Duty cycle: {}", duty_cycle);
        pwm.set_duty_cycle(duty_cycle)?;
        thread::sleep(SLEEP);

        if duty_cycle == 1.5 {
            info!("Breaking");
            break;
        }
    }

    Ok(())
}
