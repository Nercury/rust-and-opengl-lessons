extern crate gl;
extern crate sdl2;
extern crate ui;
extern crate failure;
extern crate lesson_24_x_render as render;
extern crate resources;
extern crate lesson_24_x_render_gl as render_gl;
#[macro_use]
extern crate lesson_24_x_render_gl_derive as render_gl_derive;

pub mod profiling;
pub mod debug;

use failure::err_msg;

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

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);
    gl_attr.set_accelerated_visual(true);
    gl_attr.set_double_buffer(true);

    let mut window_size = render::WindowSize {
        width: 960,
        height: 600,
        highdpi_width: 960,
        highdpi_height: 600
    };

    let window = video_subsystem
        .window("Game", window_size.width as u32, window_size.height as u32)
        .opengl()
        .resizable()
        .allow_highdpi()
        .build()?;

    println!("mutator: {:#?}", mutator);

    Ok(())
}

#[global_allocator]
#[cfg(feature = "alloc_debug")]
static GLOBAL: profiling::alloc::ProfilingAlloc = profiling::alloc::ProfilingAlloc;
