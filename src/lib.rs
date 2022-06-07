use anyhow::Result;
use cycle::get_duty_cycle;
use log::{debug, info, warn};
use pwm::{Channel, Pwm};
use std::fs::read_to_string;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

mod cycle;
pub mod logger;
pub mod pwm;

/// Returns the current temperature in celcius
fn get_temp(path: &str) -> Result<f32> {
    let contents = read_to_string(path)?;
    let millicelcius: f32 = contents.trim().parse()?;
    debug!("Temperature read in millicelcius: {}", millicelcius);

    Ok(millicelcius / 1000.0)
}

pub fn fan_loop(
    channel: Channel,
    frequency: f64,
    initial_duty_cycle: f64,
    temp_path: String,
    sleep: Duration,
) -> Result<()> {
    info!("Setting up ctrlc handler");
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        warn!("Stopping..");
        r.store(false, Ordering::SeqCst);
    })?;

    let pwm = Pwm::new(channel, frequency, initial_duty_cycle)?;

    info!("Starting temperature loop");
    let mut cycle_idx: usize = 0;
    while running.load(Ordering::SeqCst) {
        let temp = get_temp(temp_path.as_str())?;
        let (new_cycle_idx, duty_cycle) = get_duty_cycle(cycle_idx, temp)?;
        cycle_idx = new_cycle_idx;

        pwm.set_duty_cycle(duty_cycle)?;
        thread::sleep(sleep);
    }

    Ok(())
}
