#[macro_use] extern crate failure;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Failed to initialize windows: {}", _0)]
    FailedToInitializeWindows(String),
    #[fail(display = "Window height {} overflows", _0)]
    HeightOverflows(u32),
    #[fail(display = "Window width {} overflows", _0)]
    WidthOverflows(u32),
    #[fail(display = "Invalid window title")]
    InvalidTitle,
    #[fail(display = "Failed to create window: {}", _0)]
    FailedToCreateWindow(String),
}

#[derive(Debug, Copy, Clone)]
pub struct WindowDimensions {
    pub size: WindowSize,
    pub hdpi_size: WindowSize,
    pub high_dpi: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct WindowSettings {
    pub dimensions: WindowDimensions,
}

impl Default for WindowSettings {
    fn default() -> Self {
        WindowSettings {
            dimensions: WindowDimensions {
                size: WindowSize {
                    width: 960,
                    height: 600,
                },
                hdpi_size: WindowSize {
                    width: 960,
                    height: 600,
                },
                high_dpi: false,
            }
        }
    }
}

pub struct Window {
    window: sdl2::video::Window,
    settings: WindowSettings,
}

pub struct Windows {
    sdl: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
}

fn sdl_windows_err(error: String) -> Error {
    Error::FailedToInitializeWindows(error)
}

fn sdl_window_err(error: sdl2::video::WindowBuildError) -> Error {
    use sdl2::video::WindowBuildError;

    match error {
        WindowBuildError::HeightOverflows(s) => Error::HeightOverflows(s),
        WindowBuildError::WidthOverflows(s) => Error::WidthOverflows(s),
        WindowBuildError::InvalidTitle(_) => Error::InvalidTitle,
        WindowBuildError::SdlError(s) => Error::FailedToCreateWindow(s),
    }
}

impl Windows {
    pub fn new() -> Result<Windows, Error> {
        let sdl = sdl2::init().map_err(sdl_windows_err)?;
        let video = sdl.video().map_err(sdl_windows_err)?;

        Ok(Windows {
            sdl,
            video,
        })
    }

    pub fn create(&self, mut settings: WindowSettings) -> Result<Window, Error> {
        let gl_attr = self.video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 1);
        gl_attr.set_accelerated_visual(true);
        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_buffers(1);
        gl_attr.set_multisample_samples(16);

        let dims = &mut settings.dimensions;

        let mut window = self.video
            .window("Demo", dims.size.width as u32, dims.size.height as u32);
        let builder = window
            .opengl()
            .resizable();

        if dims.high_dpi {
            builder.allow_highdpi();
        }

        let mut window = builder.build().map_err(sdl_window_err)?;

        if dims.high_dpi {
            let drawable_size = window.drawable_size();
            dims.hdpi_size.width = drawable_size.0 as i32;
            dims.hdpi_size.height = drawable_size.1 as i32;
        } else {
            dims.hdpi_size.width = dims.size.width;
            dims.hdpi_size.height = dims.size.height;
        }

        let mut scale = dims.hdpi_size.width as f32 / dims.size.width as f32;
        let mut scale_modifier = 1.0;

//        let _gl_context = window.gl_create_context().map_err(sdl_err)?;
//        let gl = gl::Gl::load_with(|s| {
//            video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
//        });

        Ok(Window {
            window,
            settings,
        })
    }
}