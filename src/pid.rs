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

    /// Compute the next duty cycle given the current temperature.
    /// Returns a value clamped to 0.0–1.0.
    pub fn update(&mut self, temp: f32) -> f64 {
        let error = temp - self.target;

        self.integral += error;
        // Clamp integral to [0, limit] — negative integral is useless since duty cycle
        // can't go below 0, and letting it accumulate causes windup when temp is below target.
        let integral_limit = if self.ki > 0.0 { 1.0 / self.ki } else { 1000.0 };
        self.integral = self.integral.clamp(0.0, integral_limit);

        let derivative = error - self.prev_error;
        self.prev_error = error;

        let output = self.kp * error + self.ki * self.integral + self.kd * derivative;

        (output as f64).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_target_gives_zero() {
        let mut pid = PidController::new(55.0, 0.02, 0.001, 0.01);
        assert_eq!(pid.update(30.0), 0.0);
    }

    #[test]
    fn above_target_gives_positive() {
        let mut pid = PidController::new(55.0, 0.02, 0.001, 0.01);
        let duty = pid.update(70.0);
        assert!(duty > 0.0);
        assert!(duty <= 1.0);
    }

    #[test]
    fn way_above_target_clamps_to_one() {
        let mut pid = PidController::new(55.0, 0.02, 0.001, 0.01);
        assert_eq!(pid.update(120.0), 1.0);
    }

    #[test]
    fn integral_accumulates() {
        let mut pid = PidController::new(55.0, 0.0, 0.01, 0.0);
        // First tick: integral = 5, output = 0.05
        let d1 = pid.update(60.0);
        // Second tick: integral = 10, output = 0.1
        let d2 = pid.update(60.0);
        assert!(d2 > d1);
    }

    #[test]
    fn derivative_responds_to_rising_temp() {
        let mut pid = PidController::new(55.0, 0.0, 0.0, 0.1);
        pid.update(56.0); // error=1, deriv=1
        let d = pid.update(60.0); // error=5, deriv=4
        assert!((d - 0.4).abs() < 0.001);
    }

    #[test]
    fn no_negative_integral_windup() {
        let mut pid = PidController::new(45.0, 0.02, 0.001, 0.0);
        // Spend many ticks below target
        for _ in 0..100 {
            pid.update(37.0);
        }
        // Cross above target — should respond immediately
        let d = pid.update(46.0);
        assert!(d > 0.0, "duty cycle should be positive immediately after crossing target");
    }
}
