use na;
use std::collections::BTreeMap;
use super::buffers::LinePoint;

pub struct Container {
    pub isometry: na::Isometry3<f32>,
    pub data: Vec<LinePoint>,
}

pub struct SharedDebugLines {
    pub invalidated: bool,
    pub containers: BTreeMap<i32, Container>,
    next_id: i32,
}

impl SharedDebugLines {
    pub fn new() -> SharedDebugLines {
        SharedDebugLines {
            invalidated: true,
            containers: BTreeMap::new(),
            next_id: 0,
        }
    }

    fn get_next_id(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn new_container(&mut self, isometry: na::Isometry3<f32>, data: Vec<LinePoint>) -> i32 {
        let next_id = self.get_next_id();
        self.containers.insert(next_id, Container {
            isometry,
            data,
        });
        self.invalidated = true;
        next_id
    }

    pub fn remove_container(&mut self, key: i32) {
        self.containers.remove(&key);
        self.invalidated = true;
    }

    pub fn get_container_mut(&mut self, key: i32) -> Option<&mut Container> {
        self.invalidated = true;
        self.containers.get_mut(&key)
    }
}