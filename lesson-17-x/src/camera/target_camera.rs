use super::WasdMovement;
use nalgebra as na;

pub struct TargetCamera {
    pub target: na::Point3<f32>,
    distance: f32,
    pub rotation: na::UnitQuaternion<f32>,
    pub movement: WasdMovement,
    projection: na::Perspective3<f32>,
    invalidated: bool,
}

impl TargetCamera {
    pub fn new(
        aspect: f32,
        fov: f32,
        znear: f32,
        zfar: f32,
        _initial_tilt: f32,
        initial_distance: f32,
    ) -> TargetCamera {
        TargetCamera {
            target: na::Point3::origin(),
            distance: initial_distance,
            rotation: na::UnitQuaternion::from_axis_angle(
                &na::Vector3::x_axis(),
                ::std::f32::consts::PI / 4.0,
            ),
            movement: WasdMovement::new(),
            projection: na::Perspective3::new(aspect, fov, znear, zfar),
            invalidated: true,
        }
    }

    /// Calculate position of camera from a view matrix.
    pub fn project_pos(&self) -> na::Point3<f32> {
        na::Translation3::<f32>::from(self.target.coords)
            * self.rotation
            * na::Translation3::<f32>::from(na::Vector3::z() * self.distance)
            * na::Point3::<f32>::origin()
    }

    pub fn update_aspect(&mut self, aspect: f32) {
        self.projection.set_aspect(aspect);
    }

    pub fn get_view_matrix(&self) -> na::Matrix4<f32> {
        (na::Translation3::<f32>::from(self.target.coords)
            * self.rotation
            * na::Translation3::<f32>::from(na::Vector3::z() * self.distance)).inverse()
        .to_homogeneous()
    }

    pub fn get_p_matrix(&self) -> na::Matrix4<f32> {
        self.projection.unwrap()
    }

    pub fn get_vp_matrix(&self) -> na::Matrix4<f32> {
        self.projection.unwrap() * self.get_view_matrix()
    }

    /// Zoom scene using specified scroll wheel difference.
    pub fn zoom(&mut self, rel: f32) {
        self.distance -= rel * self.speed_from_distance();
        self.invalidated = true;
    }

    /// Rotate camera using relative mouse movement over screen pixels.
    pub fn rotate(&mut self, rel: &na::Vector2<f32>) {
        let around_x =
            na::UnitQuaternion::from_axis_angle(&na::Vector3::x_axis(), rel.y as f32 * 0.005);
        let around_z =
            na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), -rel.x as f32 * 0.005);

        self.rotation = around_z * self.rotation * around_x;

        self.invalidated = true;
    }

    /// Update camera position for the movement.
    pub fn update(&mut self, delta: f32) -> bool {
        if !self.movement.has_movement() && !self.invalidated {
            return false;
        }

        if self.movement.has_movement() {
            let mut mov3 = self.movement.get_vector();

            let camera_pos = self.project_pos();
            if camera_pos.z < self.target.z {
                mov3.y = -mov3.y;
            }

            let mov3_rotated = self.rotation * na::Vector3::new(mov3.x, mov3.y, 0.0);

            let xy = na::Vector2::new(mov3_rotated.x, mov3_rotated.y).try_normalize(0.01);

            let combined_movement = na::Vector3::new(
                xy.map(|v| v.x).unwrap_or(0.0),
                xy.map(|v| v.y).unwrap_or(0.0),
                mov3.z,
            ).try_normalize(0.01);

            if let Some(combined_movement) = combined_movement {
                let movement_translation = combined_movement
                    * (if self.movement.faster { 75.0 } else { 25.0 })
                    * delta
                    * self.speed_from_distance();

                self.target += na::Vector3::new(
                    movement_translation.x,
                    movement_translation.y,
                    movement_translation.z,
                );
            }
        }

        self.invalidated = false;

        true
    }

    pub fn speed_from_distance(&self) -> f32 {
        let min_speed = 0.1;
        let max_speed = 20.0;
        let min_distance = 1.0;
        let max_distance = 500.0;

        if self.distance > max_distance {
            max_speed
        } else if self.distance < min_distance {
            min_speed
        } else {
            (self.distance - min_distance) / (max_distance - min_distance) * (max_speed - min_speed)
                + min_speed
        }
    }
}
