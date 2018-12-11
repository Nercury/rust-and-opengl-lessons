use log::*;
use resources::{
    Resources,
    backend::FileSystem
};
use winput;
use toml_edit as toml;

mod debug;

fn main() {
    debug::init_logging();

    if let Err(e) = run() {
        error!("{}", debug::failure_to_string(e));
    }
}

struct WindowsConfig {
    width: i32,
    height: i32,
}

impl config::ConfigItem for WindowsConfig {
    fn serialize(&self, item: &mut toml::Item) {
        let table = match item.as_table_mut() {
            Some(table) => table,
            None => {
                *item = toml::table();
                item.as_table_mut().unwrap()
            }
        };

        config::ConfigItem::serialize(&self.width, &mut table.entry("width"));
        config::ConfigItem::serialize(&self.height, &mut table.entry("height"));
    }

    fn deserialize(&mut self, item: &toml::Item) {

    }
}

impl Default for WindowsConfig {
    fn default() -> Self {
        WindowsConfig {
            width: 800,
            height: 600,
        }
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
    let mut windows_config = config.pick::<WindowsConfig>(&["windows"]);
    windows_config.modify(|v| ());

    'reload: loop {
        if config.should_persist() {
            config.persist()?;
        }

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