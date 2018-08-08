use ncollide3d::bounding_volume::aabb::AABB;
use std::rc::Rc;
use std::cell::RefCell;

mod shared_selectables;
use self::shared_selectables::{SharedSelectables, ContainerHandle};

pub struct SelectableAABB {
    shared: Rc<RefCell<SharedSelectables>>,
    handle: ContainerHandle,
}

impl SelectableAABB {
    pub fn update_aabb(&self, aabb: AABB<f32>) {
        let mut shared_ref = self.shared.borrow_mut();
        if let Some(container_ref) = shared_ref.get_container_mut(&self.handle) {
            container_ref.aabb = aabb;
        }
    }
}

impl Drop for SelectableAABB {
    fn drop(&mut self) {
        let mut shared_ref = self.shared.borrow_mut();
        shared_ref.remove_from_selection(&self.handle);
        shared_ref.remove_container(&self.handle);
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

    pub fn selectable(&self, aabb: AABB<f32>) -> SelectableAABB {
        let new_handle = self.shared.borrow_mut()
            .new_container(aabb);

        SelectableAABB {
            shared: self.shared.clone(),
            handle: new_handle,
        }
    }
}