use anyhow::Result;
use log::{debug, info, warn};
use pid::PidController;
use pwm::{Channel, Pwm};
use std::fs::read_to_string;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

pub mod logger;
mod pid;
pub mod pwm;

/// Returns the current temperature in celsius
fn get_temp(path: &str) -> Result<f32> {
    let contents = read_to_string(path)?;
    let millicelsius: f32 = contents.trim().parse()?;
    debug!("Temperature read in millicelsius: {}", millicelsius);

    Ok(millicelsius / 1000.0)
}

pub fn fan_loop(
    channel: Channel,
    frequency: f64,
    initial_duty_cycle: f64,
    temp_path: String,
    sleep: Duration,
    target_temp: f32,
    kp: f32,
    ki: f32,
    kd: f32,
) -> Result<()> {
    info!("Setting up ctrlc handler");
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        warn!("Stopping..");
        r.store(false, Ordering::SeqCst);
    })?;

    let pwm = Pwm::new(channel, frequency, initial_duty_cycle)?;
    let mut pid = PidController::new(target_temp, kp, ki, kd);

    info!(
        "Starting PID fan control (target {}°C, Kp={}, Ki={}, Kd={})",
        target_temp, kp, ki, kd
    );
    while running.load(Ordering::SeqCst) {
        let temp = get_temp(temp_path.as_str())?;
        let duty_cycle = pid.update(temp);

        debug!("{:.1}°C → duty cycle {:.3}", temp, duty_cycle);
        pwm.set_duty_cycle(duty_cycle)?;
        thread::sleep(sleep);
    }

    info!("Setting fan to 100% before exit");
    pwm.set_duty_cycle(1.0)?;

    Ok(())
}
