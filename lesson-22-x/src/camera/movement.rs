use nalgebra as na;

pub struct WasdMovement {
    pub left: bool,
    pub forward: bool,
    pub backward: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub faster: bool,
}

impl WasdMovement {
    pub fn new() -> WasdMovement {
        WasdMovement {
            left: false,
            forward: false,
            backward: false,
            right: false,
            up: false,
            down: false,
            faster: false,
        }
    }

    /// Getting Vector2 for movement would result in non-zero movement.
    pub fn has_movement(&self) -> bool {
        self.left || self.right || self.forward || self.backward || self.up || self.down
    }

    /// Get movement vector on a horizontal plane. The Y axis is forward, the Z is up.
    pub fn get_vector(&self) -> na::Vector3<f32> {
        let mut x = 0.0;
        if self.right {
            x += 1.0;
        }
        if self.left {
            x -= 1.0;
        }

        let mut y = 0.0;
        if self.forward {
            y += 1.0;
        }
        if self.backward {
            y -= 1.0;
        }

        let mut z = 0.0;
        if self.up {
            z += 1.0;
        }
        if self.down {
            z -= 1.0;
        }

        na::Vector3::new(x, y, z)
    }
}
