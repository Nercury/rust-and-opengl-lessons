use ncollide3d::bounding_volume::aabb::AABB;
use ncollide3d::query::Ray;
use nalgebra as na;
use std::rc::Rc;
use std::cell::RefCell;

mod shared_selectables;
use self::shared_selectables::{SharedSelectables, ContainerHandle, Container};

#[derive(Copy, Clone, PartialEq)]
pub enum Action {
    Click,
    Drag { new_isometry: na::Isometry3<f32> },
}

pub struct SelectableAABB {
    shared: Rc<RefCell<SharedSelectables>>,
    handle: ContainerHandle,
}

impl SelectableAABB {
    pub fn has_handle(&self, handle: ContainerHandle) -> bool {
        self.handle == handle
    }

    pub fn update_aabb(&self, aabb: AABB<f32>) {
        let mut shared_ref = self.shared.borrow_mut();
        if let Some(container_ref) = shared_ref.get_container_mut(self.handle) {
            container_ref.aabb = aabb;
        }
    }

    pub fn update_isometry(&self, isometry: na::Isometry3<f32>) {
        let mut shared_ref = self.shared.borrow_mut();
        if let Some(container_ref) = shared_ref.get_container_mut(self.handle) {
            container_ref.isometry = isometry;
        }
    }

    pub fn drain_pending_action(&self) -> Option<Action> {
        self.shared.borrow_mut().drain_pending_action(self.handle)
    }

    pub fn select(&self) {
        self.shared.borrow_mut().select(self.handle)
    }
}

impl Drop for SelectableAABB {
    fn drop(&mut self) {
        let mut shared_ref = self.shared.borrow_mut();
        shared_ref.remove_container(self.handle);
    }
}

pub struct Selectables {
    shared: Rc<RefCell<SharedSelectables>>,
}

impl Selectables {
    pub fn new() -> Selectables {
        Selectables {
            shared: Rc::new(RefCell::new(SharedSelectables::new())),
        }
    }

    pub fn selectable(&self, aabb: AABB<f32>, isometry: na::Isometry3<f32>) -> SelectableAABB {
        let new_handle = self.shared.borrow_mut()
            .new_container(aabb, isometry);

        SelectableAABB {
            shared: self.shared.clone(),
            handle: new_handle,
        }
    }

    pub fn cast_cursor(&self, ray: &Ray<f32>, camera_dir: &na::Vector3<f32>) {
        self.shared.borrow_mut().cast_cursor(ray, camera_dir);
    }

    pub fn send_mouse_down(&self) {
        self.shared.borrow_mut().send_mouse_down();
    }

    pub fn send_mouse_up(&self) {
        self.shared.borrow_mut().send_mouse_up();
    }

    pub fn cancel_drag(&self) {
        self.shared.borrow_mut().cancel_drag();
    }

    pub fn get_hover_aabb(&self) -> Option<(ContainerHandle, Container)> {
        self.shared.borrow().get_hover_aabb()
    }

    pub fn get_selected_aabb(&self) -> Option<(ContainerHandle, Container)> {
        self.shared.borrow().get_selected_aabb()
    }
}