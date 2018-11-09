use na::Vector3;

pub fn color_red() -> Vector3<f32> {
    Vector3::<f32>::new(1.0, 0.0, 0.0)
}

pub fn color_green() -> Vector3<f32> {
    Vector3::<f32>::new(0.0, 1.0, 0.0)
}

pub fn color_yellow() -> Vector3<f32> {
    Vector3::<f32>::new(1.0, 1.0, 0.0)
}

pub fn color_light_blue() -> Vector3<f32> {
    Vector3::<f32>::new(0.7, 0.7, 1.0)
}

pub fn color_blue() -> Vector3<f32> {
    Vector3::<f32>::new(0.0, 0.0, 1.0)
}

pub fn color_white() -> Vector3<f32> {
    Vector3::<f32>::new(1.0, 1.0, 1.0)
}

pub fn color_black() -> Vector3<f32> {
    Vector3::<f32>::new(0.0, 0.0, 0.0)
}

pub fn color_gray() -> Vector3<f32> {
    Vector3::<f32>::new(0.5, 0.5, 0.5)
}

pub struct WindowSize {
    pub width: i32,
    pub height: i32,
    pub highdpi_width: i32,
    pub highdpi_height: i32,
}
