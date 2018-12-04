use log::*;
use resources::{
    Resources,
    backend::FileSystem
};
use winput;

mod debug;

fn main() {
    debug::init_logging();

    if let Err(e) = run() {
        error!("{}", debug::failure_to_string(e));
    }
}

struct WindowsConfig {

}

impl Default for WindowsConfig {
    fn default() -> Self {
        WindowsConfig {}
    }
}

fn run() -> Result<(), failure::Error> {
    let resources = Resources::new().loaded_from(
        "core",
        0,
        FileSystem::from_rel_path(env!("CARGO_MANIFEST_DIR"), "core")
            .with_write()
            .with_watch(),
    );

    let config = config::Config::new(resources.resource("Windows.toml"));
    let windows_config = config.pick::<WindowsConfig>("windows");

    'reload: loop {



//        let windows = winput::Windows::new()?;
//        let window = windows.create(winput::WindowSettings::default());
//        let window = windows.create(winput::WindowSettings::default());
//
//        let res = resources.resource("shaders/quad.frag");
//
//        loop {
//            if let Some(p) = resources.new_changes() {
//                println!("res: {}", String::from_utf8_lossy(&res.get().unwrap()));
//                resources.notify_changes_synced(p);
//            }
//
//            ::std::thread::sleep_ms(500);
//        }
    }

    Ok(())
}