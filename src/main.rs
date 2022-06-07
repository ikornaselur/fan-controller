use fan_controller::{fan_loop, logger::Logger};
use log::LevelFilter;
use std::error::Error;

static LOGGER: Logger = Logger;

fn main() -> Result<(), Box<dyn Error>> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();

    fan_loop()?;

    Ok(())
}
