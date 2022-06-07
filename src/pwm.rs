use log::debug;
#[cfg(target_arch = "aarch64")]
use rppal;
use std::error::Error;

#[derive(Debug)]
pub enum Channel {
    Pwm0,
    Pwm1,
}

pub struct Pwm {
    #[cfg(target_arch = "aarch64")]
    rppal_pwm: rppal::pwm::Pwm,
}

#[cfg(target_arch = "aarch64")]
impl Pwm {
    pub fn new(channel: Channel, frequency: f64, duty_cycle: f64) -> Result<Self, Box<dyn Error>> {
        debug!(
            "Initialising PWM with {:?} {:?} {:?}",
            channel, frequency, duty_cycle
        );
        let rppal_channel = match channel {
            Channel::Pwm0 => rppal::pwm::Channel::Pwm0,
            Channel::Pwm1 => rppal::pwm::Channel::Pwm1,
        };
        Ok(Self {
            rppal_pwm: rppal::pwm::Pwm::with_frequency(
                rppal_channel,
                frequency,
                duty_cycle,
                rppal::pwm::Polarity::Normal,
                true,
            )
            .unwrap(),
        })
    }

    pub fn set_duty_cycle(&self, duty_cycle: f64) -> Result<(), Box<dyn Error>> {
        debug!("Setting duty cycle to {}", duty_cycle);
        self.rppal_pwm.set_duty_cycle(duty_cycle).unwrap();
        Ok(())
    }
}

#[cfg(target_arch = "x86_64")]
impl Pwm {
    pub fn new(channel: Channel, frequency: f64, duty_cycle: f64) -> Result<Self, Box<dyn Error>> {
        debug!(
            "Creating new fake PWM with {:?} {:?} {:?}",
            channel, frequency, duty_cycle
        );
        Ok(Self {})
    }

    pub fn set_duty_cycle(&self, duty_cycle: f64) -> Result<(), Box<dyn Error>> {
        debug!("Setting duty cycle to {}", duty_cycle);
        Ok(())
    }
}
