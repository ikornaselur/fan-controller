#[derive(Debug)]
pub struct PidOutput {
    pub duty_cycle: f64,
    pub error: f32,
    pub integral: f32,
    pub derivative: f32,
}

pub struct PidController {
    target: f32,
    kp: f32,
    ki: f32,
    kd: f32,
    integral: f32,
    prev_error: f32,
}

impl PidController {
    pub fn new(target: f32, kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            target,
            kp,
            ki,
            kd,
            integral: 0.0,
            prev_error: 0.0,
        }
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn kp(&self) -> f32 {
        self.kp
    }

    pub fn ki(&self) -> f32 {
        self.ki
    }

    pub fn kd(&self) -> f32 {
        self.kd
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn set_kp(&mut self, kp: f32) {
        self.kp = kp;
        self.integral = 0.0;
    }

    pub fn set_ki(&mut self, ki: f32) {
        self.ki = ki;
        self.integral = 0.0;
    }

    pub fn set_kd(&mut self, kd: f32) {
        self.kd = kd;
    }

    /// Compute the next duty cycle given the current temperature.
    pub fn update(&mut self, temp: f32) -> PidOutput {
        let error = temp - self.target;

        if error > 0.0 {
            self.integral += error;
        } else {
            // Decay integral when below target to prevent overshoot oscillation
            self.integral *= 0.95;
        }
        let integral_limit = if self.ki > 0.0 { 1.0 / self.ki } else { 1000.0 };
        self.integral = self.integral.clamp(0.0, integral_limit);

        let derivative = error - self.prev_error;
        self.prev_error = error;

        let output = self.kp * error + self.ki * self.integral + self.kd * derivative;

        PidOutput {
            duty_cycle: (output as f64).clamp(0.0, 1.0),
            error,
            integral: self.integral,
            derivative,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_target_gives_zero() {
        let mut pid = PidController::new(55.0, 0.02, 0.001, 0.01);
        assert_eq!(pid.update(30.0).duty_cycle, 0.0);
    }

    #[test]
    fn above_target_gives_positive() {
        let mut pid = PidController::new(55.0, 0.02, 0.001, 0.01);
        let out = pid.update(70.0);
        assert!(out.duty_cycle > 0.0);
        assert!(out.duty_cycle <= 1.0);
    }

    #[test]
    fn way_above_target_clamps_to_one() {
        let mut pid = PidController::new(55.0, 0.02, 0.001, 0.01);
        assert_eq!(pid.update(120.0).duty_cycle, 1.0);
    }

    #[test]
    fn integral_accumulates() {
        let mut pid = PidController::new(55.0, 0.0, 0.01, 0.0);
        let d1 = pid.update(60.0).duty_cycle;
        let d2 = pid.update(60.0).duty_cycle;
        assert!(d2 > d1);
    }

    #[test]
    fn derivative_responds_to_rising_temp() {
        let mut pid = PidController::new(55.0, 0.0, 0.0, 0.1);
        pid.update(56.0);
        let d = pid.update(60.0).duty_cycle;
        assert!((d - 0.4).abs() < 0.001);
    }

    #[test]
    fn no_negative_integral_windup() {
        let mut pid = PidController::new(45.0, 0.02, 0.001, 0.0);
        for _ in 0..100 {
            pid.update(37.0);
        }
        let d = pid.update(46.0).duty_cycle;
        assert!(d > 0.0, "duty cycle should be positive immediately after crossing target");
    }
}
