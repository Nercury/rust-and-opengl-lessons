use ncollide3d::bounding_volume::aabb::AABB;
use ncollide3d::query::RayCast;
use slab::Slab;

pub struct SharedSelectables {
    containers: Slab<Container>,
    selected: Option<ContainerHandle>,
}

impl SharedSelectables {
    pub fn new() -> SharedSelectables {
        SharedSelectables {
            containers: Slab::new(),
            selected: None,
        }
    }

    pub fn new_container(&mut self, aabb: AABB<f32>) -> ContainerHandle {
        ContainerHandle(
            self.containers.insert(Container { aabb })
        )
    }

    pub fn remove_container(&mut self, handle: &ContainerHandle) {
        self.containers.remove(handle.0);
    }

    pub fn get_container_mut(&mut self, handle: &ContainerHandle) -> Option<&mut Container> {
        self.containers.get_mut(handle.0)
    }

    pub fn remove_from_selection(&mut self, handle: &ContainerHandle) {
        if self.selected.as_ref() == Some(handle) {
            self.selected = None;
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct ContainerHandle(usize);

pub struct Container {
    pub aabb: AABB<f32>,
}