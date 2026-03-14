use anyhow::Result;
use log::{debug, info, warn};
use pid::PidController;
use pwm::{Channel, Pwm};
use std::collections::VecDeque;
use std::fs::read_to_string;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

pub mod logger;
pub mod mqtt;
pub mod pid;
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
    mqtt_config: Option<mqtt::MqttConfig>,
    temp_samples: usize,
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
    let mut temp_buffer: VecDeque<f32> = VecDeque::with_capacity(temp_samples);

    let mut mqtt = if let Some(config) = mqtt_config {
        let mut handle = mqtt::MqttHandle::new(config)?;
        handle.publish_discovery(&pid)?;
        Some(handle)
    } else {
        None
    };

    info!(
        "Starting PID fan control (target {}°C, Kp={}, Ki={}, Kd={}, samples={})",
        target_temp, kp, ki, kd, temp_samples
    );
    while running.load(Ordering::SeqCst) {
        let raw_temp = get_temp(temp_path.as_str())?;

        if temp_buffer.len() >= temp_samples {
            temp_buffer.pop_front();
        }
        temp_buffer.push_back(raw_temp);
        let temp = temp_buffer.iter().sum::<f32>() / temp_buffer.len() as f32;

        let output = pid.update(temp);

        debug!("{:.1}°C → duty cycle {:.3}", temp, output.duty_cycle);
        pwm.set_duty_cycle(output.duty_cycle)?;

        if let Some(ref mut handle) = mqtt {
            if let Err(e) = handle.publish_state(temp, &output) {
                warn!("MQTT publish failed: {}", e);
            }
            handle.poll(sleep, &mut pid, &running);
        } else {
            thread::sleep(sleep);
        }
    }

    if let Some(ref mut handle) = mqtt {
        info!("Disconnecting MQTT");
        handle.shutdown();
    }

    info!("Setting fan to 100% before exit");
    pwm.set_duty_cycle(1.0)?;

    Ok(())
}
