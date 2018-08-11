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

#[derive(Clone)]
enum DragState {
    NoObject,
    ViewPlane {
        handle: ContainerHandle,
        initial_isometry: na::Isometry3<f32>,
        point: na::Point3<f32>,
    },
}

pub struct SharedSelectables {
    containers: Slab<Container>,
    under_cursor: Option<ContainerHandle>,
    selected: Option<ContainerHandle>,
    query: Option<PendingAction>,

    mouse_down: bool,
    drag_state: Option<DragState>,
}

impl SharedSelectables {
    pub fn new() -> SharedSelectables {
        SharedSelectables {
            containers: Slab::new(),
            under_cursor: None,
            selected: None,
            query: None,

            mouse_down: false,
            drag_state: None,
        }
    }

    pub fn new_container(&mut self, aabb: AABB<f32>, isometry: na::Isometry3<f32>) -> ContainerHandle {
        ContainerHandle(
            self.containers.insert(Container {
                aabb,
                isometry,
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

    pub fn cast_cursor(&mut self, ray: &Ray<f32>, camera_dir: &na::Vector3<f32>) {
        let mut closest = None;
        let mut impact_point = None;
        let mut impact_obj_isometry = None;
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
                    impact_obj_isometry = Some(c.isometry);
                    closest = Some(handle);
                }
            }
        }

        self.under_cursor = closest.map(ContainerHandle);

        match self.drag_state {
            None => if self.mouse_down {
                match (self.under_cursor, impact_point, impact_obj_isometry) {
                    (Some(under_cursor_obj), Some(start_point), Some(impact_obj_isometry)) => {
                        self.drag_state = Some(DragState::ViewPlane {
                            handle: under_cursor_obj,
                            initial_isometry: impact_obj_isometry,
                            point: start_point,
                        })
                    }
                    (None, _, _) => self.drag_state = Some(DragState::NoObject), // dragging empty space until mouse up
                    _ => (),
                }
            },
            Some(DragState::ViewPlane { handle, initial_isometry, point }) => {
                let plane = Plane::new(na::Unit::new_normalize(-camera_dir));
                let plane_isometry = na::Isometry3::from_parts(
                    na::Translation3::from_vector(point.coords),
                    na::UnitQuaternion::identity(),
                );
                if let Some(toi) = plane.toi_with_ray(&plane_isometry, ray, true) {
                    let dragged_to_point_on_place = ray.origin + ray.dir * toi;
                    let drag_vector = dragged_to_point_on_place - point;
                    if na::norm_squared(&drag_vector) > 0.1 * 0.1 {
                        self.query = Some(PendingAction {
                            handle: handle,
                            action: Action::Drag {
                                new_isometry: na::Isometry3::from_parts(
                                    na::Translation3::from_vector(drag_vector),
                                    na::UnitQuaternion::identity(),
                                ) * initial_isometry
                            },
                        });
                    } else {
                        self.query = Some(PendingAction {
                            handle: handle,
                            action: Action::Drag { new_isometry: initial_isometry },
                        });
                    }
                }
            }
            _ => (),
        };
    }

    pub fn send_mouse_down(&mut self) {
        self.mouse_down = true;
        if self.selected.is_some() && self.under_cursor.is_none() {
            self.selected = None;
        }
    }

    pub fn send_mouse_up(&mut self) {
        self.mouse_down = false;
        self.drag_state = None;
    }

    pub fn cancel_drag(&mut self) {
        match self.drag_state {
            Some(DragState::ViewPlane { handle, initial_isometry, .. }) => {
                self.drag_state = Some(DragState::NoObject);
                self.query = Some(PendingAction {
                    handle,
                    action: Action::Drag { new_isometry: initial_isometry },
                });
            },
            _ => (),
        }
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