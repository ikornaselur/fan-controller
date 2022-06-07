use cycle::get_duty_cycle;
use log::{debug, info, warn};
use pwm::{Channel, Pwm};
use std::fs::read_to_string;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{error::Error, thread, time::Duration};

mod cycle;
pub mod logger;
mod pwm;

const SLEEP: Duration = Duration::from_secs(1);
const FREQUENCY: f64 = 25_000.0; // 25kHz for Noctua fans
const INITIAL_DUTY_CYCLE: f64 = 1.0;
const TEMP_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";

/// Returns the current temperature in celcius
fn get_temp() -> Result<f32, Box<dyn Error>> {
    let contents = read_to_string(TEMP_PATH)?;
    let millicelcius: f32 = contents.trim().parse()?;
    debug!("Temperature read in millicelcius: {}", millicelcius);

    Ok(millicelcius / 1000.0)
}

pub fn fan_loop() -> Result<(), Box<dyn Error>> {
    info!("Setting up ctrlc handler");
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        warn!("Stopping..");
        r.store(false, Ordering::SeqCst);
    })?;

    let pwm = Pwm::new(Channel::Pwm0, FREQUENCY, INITIAL_DUTY_CYCLE)?;

    info!("Starting temperature loop");
    let mut cycle_idx: usize = 0;
    while running.load(Ordering::SeqCst) {
        let temp = get_temp()?;
        let (new_cycle_idx, duty_cycle) = get_duty_cycle(cycle_idx, temp)?;
        cycle_idx = new_cycle_idx;

        pwm.set_duty_cycle(duty_cycle)?;
        thread::sleep(SLEEP);
    }

    Ok(())
}
