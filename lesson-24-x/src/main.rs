extern crate gl;
extern crate ui;
extern crate failure;
extern crate resources;
extern crate lesson_24_x_render_gl;
#[macro_use]
extern crate lesson_24_x_render_gl_derive;

pub mod profiling;
pub mod debug;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let resources = resources::Resources::new()
        .loaded_from(
            "core", 0,
            resources::backend::FileSystem::from_rel_path(env!("CARGO_MANIFEST_DIR"), "core")
                .with_write()
                .with_watch(),
        );

    let config_resource = resources.resource("Config.toml");
    let config = config_resource.get().unwrap();

    let schema = ui::schema::Root::new(ui::Size::new(800.0.into(), 500.0.into()))
        .with_container(
            ui::schema::Container::PaneLeft(
                ui::schema::PaneLeft::new(40.0.into())
                    .with_bg_color(ui::Color::new(0.5, 0.1, 0.1))
            )
        );

    let mut mutator = ui::mutator::Mutator::from_schema(&schema);

    println!("mutator: {:#?}", mutator);

    Ok(())
}

#[global_allocator]
#[cfg(feature = "alloc_debug")]
static GLOBAL: profiling::alloc::ProfilingAlloc = profiling::alloc::ProfilingAlloc;
