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