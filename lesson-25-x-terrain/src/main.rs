use log::*;

mod debug;

fn main() {
    debug::init_logging();

    if let Err(e) = run() {
        error!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    info!("hello");
    Ok(())
}