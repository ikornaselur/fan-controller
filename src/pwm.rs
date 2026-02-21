use anyhow::Result;
use log::debug;
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
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    rppal_pwm: rppal::pwm::Pwm,
}

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
impl Pwm {
    /// Find the hardware PWM chip by skipping any firmware-claimed chips.
    fn find_hardware_pwm_chip() -> Result<u8> {
        for chip in 0..8 {
            let device_link = format!("/sys/class/pwm/pwmchip{}/device", chip);
            if let Ok(target) = std::fs::read_link(&device_link) {
                let target_str = target.to_string_lossy();
                if !target_str.contains("firmware") {
                    debug!("Using pwmchip{} ({})", chip, target_str);
                    return Ok(chip);
                }
                debug!("Skipping pwmchip{} (firmware)", chip);
            }
        }
        anyhow::bail!("No hardware PWM chip found in /sys/class/pwm/")
    }

    pub fn new(channel: Channel, frequency: f64, duty_cycle: f64) -> Result<Self> {
        debug!(
            "Initialising PWM with {:?} {:?} {:?}",
            channel, frequency, duty_cycle
        );
        let channel_index = match channel {
            Channel::Pwm0 => 0,
            Channel::Pwm1 => 1,
        };
        let chip = Self::find_hardware_pwm_chip()?;
        let mut rppal_pwm = rppal::pwm::Pwm::with_pwmchip(chip, channel_index)?;
        rppal_pwm.set_reset_on_drop(false);
        rppal_pwm.set_polarity(rppal::pwm::Polarity::Normal)?;
        rppal_pwm.set_frequency(frequency, duty_cycle)?;
        rppal_pwm.enable()?;
        Ok(Self { rppal_pwm })
    }

    pub fn set_duty_cycle(&self, duty_cycle: f64) -> Result<()> {
        debug!("Setting duty cycle to {}", duty_cycle);
        self.rppal_pwm
            .set_duty_cycle(duty_cycle)
            .map_err(|err| err.into())
    }
}

#[cfg(not(all(target_arch = "aarch64", target_os = "linux")))]
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
