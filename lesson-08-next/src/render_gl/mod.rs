mod shader;

pub use self::shader::{Shader, Program, Error};

#[allow(non_camel_case_types)]
pub mod format {
    use std::mem;

    pub trait Format {
        const BYTES: usize;
    }

    #[repr(C, packed)]
    pub struct f32_f32_f32(f32, f32, f32);
    #[repr(C, packed)]
    pub struct f32_f32_f32_f32(f32, f32, f32, f32);
    #[repr(C, packed)]
    pub struct u2_u10_u10_u10_rev(u32);

    impl Format for f32_f32_f32 {
        const BYTES: usize = mem::size_of::<Self>();
    }

    impl Format for f32_f32_f32_f32 {
        const BYTES: usize = mem::size_of::<Self>();
    }

    impl Format for u2_u10_u10_u10_rev {
        const BYTES: usize = mem::size_of::<Self>();
    }
}

#[repr(C, packed)]
pub struct TriangleData {
    position: format::f32_f32_f32,
    color: format::f32_f32_f32,
}