use log::{debug, info};
use std::error::Error;

const CYCLES: [(f64, f32, f32); 12] = [
    (0.0, 0.0, 30.0),
    (0.2, 30.0, 35.0),
    (0.4, 35.0, 40.0),
    (0.6, 40.0, 45.0),
    (0.65, 45.0, 50.0),
    (0.7, 50.0, 55.0),
    (0.75, 55.0, 60.0),
    (0.8, 60.0, 65.0),
    (0.85, 65.0, 70.0),
    (0.9, 70.0, 75.0),
    (0.95, 75.0, 80.0),
    (1.0, 80.0, 256.0),
];
const BUFFER: f32 = 0.3;

pub fn get_duty_cycle(cycle_idx: usize, temp: f32) -> Result<(usize, f64), Box<dyn Error>> {
    let (cycle, min_temp, max_temp) = CYCLES[cycle_idx];

    if temp >= max_temp {
        // We should move up to next tier
        let new_cycle = CYCLES[cycle_idx + 1].0;
        info!("Moving up to next tier with {} duty cycle", new_cycle);
        Ok((cycle_idx + 1, new_cycle))
    } else if temp < min_temp {
        // Check if we're more than 30% into previous cycle before moving, to prevent ping/ponging
        // back and forth
        let (p_cycle, p_min_temp, p_max_temp) = CYCLES[cycle_idx - 1];
        let p_range = p_max_temp - p_min_temp;
        let buffer = p_range * BUFFER;
        if temp < min_temp - buffer {
            // We've gone far enough into previous cycle
            info!("Moving down to previous tier with {} duty cycle", p_cycle);
            Ok((cycle_idx - 1, p_cycle))
        } else {
            // We're within the buffer
            debug!("Need to be below {}°C to move down", min_temp - buffer);
            Ok((cycle_idx, cycle))
        }
    } else {
        // We're still just within the current cycle
        Ok((cycle_idx, cycle))
    }
}
