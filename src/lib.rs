use cycle::get_duty_cycle;
use log::{debug, info, warn};
#[cfg(target_arch = "aarch64")]
use rppal::pwm::{Channel, Polarity, Pwm};
use std::fs::read_to_string;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{error::Error, thread, time::Duration};

mod cycle;
pub mod logger;

const SLEEP: Duration = Duration::from_secs(1);
#[cfg(target_arch = "aarch64")]
const FREQUENCY: f64 = 25_000.0; // 25kHz for Noctua fans
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

    info!("Initialising pwm");
    #[cfg(target_arch = "aarch64")]
    let pwm = Pwm::with_frequency(Channel::Pwm0, FREQUENCY, 1.0, Polarity::Normal, true)?;

    info!("Starting temperature loop");
    let mut cycle_idx: usize = 0;
    while running.load(Ordering::SeqCst) {
        let temp = get_temp()?;
        let (new_cycle_idx, duty_cycle) = get_duty_cycle(cycle_idx, temp)?;
        cycle_idx = new_cycle_idx;

        debug!("Duty cycle: {}", duty_cycle);
        #[cfg(target_arch = "aarch64")]
        pwm.set_duty_cycle(duty_cycle)?;
        thread::sleep(SLEEP);
    }

    Ok(())
}
