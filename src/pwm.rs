use anyhow::Result;
use log::debug;
#[cfg(target_arch = "aarch64")]
use rppal;
use std::fmt;

#[derive(Debug)]
pub enum Channel {
    Pwm0,
    Pwm1,
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Channel PWM{}",
            match self {
                Channel::Pwm0 => "0",
                Channel::Pwm1 => "1",
            }
        )
    }
}

pub struct Pwm {
    #[cfg(target_arch = "aarch64")]
    rppal_pwm: rppal::pwm::Pwm,
}

#[cfg(target_arch = "aarch64")]
impl Pwm {
    pub fn new(channel: Channel, frequency: f64, duty_cycle: f64) -> Result<Self> {
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
            )?,
        })
    }

    pub fn set_duty_cycle(&self, duty_cycle: f64) -> Result<()> {
        debug!("Setting duty cycle to {}", duty_cycle);
        self.rppal_pwm
            .set_duty_cycle(duty_cycle)
            .map_err(|err| err.into())
    }
}

#[cfg(target_arch = "x86_64")]
impl Pwm {
    pub fn new(channel: Channel, frequency: f64, duty_cycle: f64) -> Result<Self> {
        debug!(
            "Creating new fake PWM with {:?} {:?} {:?}",
            channel, frequency, duty_cycle
        );
        Ok(Self {})
    }

    pub fn set_duty_cycle(&self, duty_cycle: f64) -> Result<()> {
        debug!("Setting duty cycle to {}", duty_cycle);
        Ok(())
    }
}
