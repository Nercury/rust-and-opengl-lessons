use ncollide3d::bounding_volume::aabb::AABB;
use ncollide3d::shape::Plane;
use ncollide3d::query::{RayCast, Ray};
use nalgebra as na;
use slab::Slab;
use super::Action;

#[derive(Copy, Clone)]
struct PendingAction {
    handle: ContainerHandle,
    action: Action,
}

pub struct SharedSelectables {
    containers: Slab<Container>,
    under_cursor: Option<ContainerHandle>,
    selected: Option<ContainerHandle>,
    dragged: Option<ContainerHandle>,
    query: Option<PendingAction>,

    mouse_down: bool,
    accumulated_motion: na::Vector2<f32>,
    drag_started: bool,

    min_distance2_for_drag: f32,
    start_drag_point: Option<na::Point3<f32>>,
}

impl SharedSelectables {
    pub fn new() -> SharedSelectables {
        SharedSelectables {
            containers: Slab::new(),
            under_cursor: None,
            selected: None,
            dragged: None,
            query: None,

            mouse_down: false,
            accumulated_motion: na::zero(),
            drag_started: false,

            min_distance2_for_drag: (1.0 / 100.0) * (1.0 / 100.0),
            start_drag_point: None,
        }
    }

    pub fn new_container(&mut self, aabb: AABB<f32>, isometry: na::Isometry3<f32>) -> ContainerHandle {
        ContainerHandle(
            self.containers.insert(Container {
                aabb, isometry
            })
        )
    }

    pub fn remove_container(&mut self, handle: ContainerHandle) {
        self.containers.remove(handle.0);
    }

    pub fn get_container_mut(&mut self, handle: ContainerHandle) -> Option<&mut Container> {
        self.containers.get_mut(handle.0)
    }

    pub fn remove_from_selection(&mut self, handle: ContainerHandle) {
        if self.under_cursor == Some(handle) {
            self.under_cursor = None;
        }
    }

    pub fn cast_cursor(&mut self, ray: &Ray<f32>, rel_motion: &na::Vector2<f32>) {
        let mut closest = None;
        let mut impact_point = None;
        let mut closest_distance2 = None;

        for (handle, c) in &self.containers {
            if let Some(toi) = c.aabb.toi_with_ray(&c.isometry, ray, true) {
                let point = ray.origin + ray.dir * toi;
                let distance2 = na::distance_squared(&point, &ray.origin);
                let new_closest = match closest_distance2 {
                    None => true,
                    Some(ref cd) => if distance2 < *cd { true } else { false },
                };

                if new_closest {
                    closest_distance2 = Some(distance2);
                    impact_point = Some(point);
                    closest = Some(handle);
                }
            }
        }

        self.under_cursor = closest.map(ContainerHandle);

        let drag_object = if self.drag_started {
            self.dragged
        } else {
            self.under_cursor
        };

        if let Some(drag_object) = drag_object {
            if let (true, Some(start_drag_point)) = (self.mouse_down, self.start_drag_point) {
                self.dragged = Some(drag_object);
                self.accumulated_motion += rel_motion;
                if na::norm_squared(&self.accumulated_motion) > self.min_distance2_for_drag || self.drag_started {

                    let movement_plane = Plane::new(na::Unit::new_normalize(-ray.dir));
                    let plane_position = na::Isometry3::from_parts(na::Translation3::from_vector(start_drag_point.coords), na::UnitQuaternion::identity());
                    if let Some(toi) = movement_plane.toi_with_ray(&plane_position, ray, true) {
                        let dragged_to_point_on_place = ray.origin + ray.dir * toi;
                        self.query = Some(PendingAction {
                            handle: drag_object,
                            action: Action::Drag {
                                diff: na::Isometry3::from_parts(
                                    na::Translation3::from_vector(dragged_to_point_on_place - start_drag_point),
                                    na::UnitQuaternion::identity()
                                )
                            }
                        });
                        self.accumulated_motion = na::zero();
                        self.start_drag_point = Some(impact_point.unwrap_or(dragged_to_point_on_place));
                    }

                    self.drag_started = true;
                }
            } else {
                self.accumulated_motion = na::zero();
                self.start_drag_point = impact_point;
            }
        } else {
            if self.mouse_down {
                self.drag_started = true; // dragging empty space until mouse up
            }
        }
    }

    pub fn send_mouse_down(&mut self) {
        self.mouse_down = true;
        if self.selected.is_some() && self.under_cursor.is_none() {
            self.selected = None;
        }
    }

    pub fn send_mouse_up(&mut self) {
        self.mouse_down = false;
        self.dragged = None;
        self.start_drag_point = None;
        self.drag_started = false;
    }

    pub fn take_pending_action(&mut self, consumer_handle: ContainerHandle) -> Option<Action> {
        if let Some(PendingAction { handle, .. }) = self.query {
            if consumer_handle == handle {
                return self.query.take().map(|p| p.action);
            }
        }

        None
    }

    pub fn select(&mut self, handle: ContainerHandle) {
        self.selected = Some(handle);
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ContainerHandle(usize);

pub struct Container {
    pub aabb: AABB<f32>,
    pub isometry: na::Isometry3<f32>,
}