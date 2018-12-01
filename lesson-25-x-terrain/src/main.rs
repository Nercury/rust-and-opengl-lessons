use log::*;
use resources::{
    Resources,
    backend::FileSystem
};

mod debug;
mod onion;

fn main() {
    debug::init_logging();

    if let Err(e) = run() {
        error!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let resources = Resources::new().loaded_from(
        "core",
        0,
        FileSystem::from_rel_path(env!("CARGO_MANIFEST_DIR"), "core")
            .with_write(),
    );

    let res = resources.resource("shaders/quad.frag");

    loop {
        if let Some(p) = resources.new_changes() {
            println!("res: {}", String::from_utf8_lossy(&res.get().unwrap()));
            resources.notify_changes_synced(p);
        }

        let mut v = String::new();
        ::std::io::stdin().read_line(&mut v).unwrap();
    }

    Ok(())
}