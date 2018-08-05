use std::collections::BTreeMap;
use super::LinePoint;

pub struct SharedDebugLines {
    pub invalidated: bool,
    pub containers: BTreeMap<i32, Vec<LinePoint>>,
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

    pub fn new_container(&mut self, data: Vec<LinePoint>) -> i32 {
        let next_id = self.get_next_id();
        self.containers.insert(next_id, data);
        self.invalidated = true;
        next_id
    }

    pub fn remove_container(&mut self, key: i32) {
        self.containers.remove(&key);
        self.invalidated = true;
    }

    pub fn get_container_mut(&mut self, key: i32) -> Option<&mut [LinePoint]> {
        self.invalidated = true;
        self.containers.get_mut(&key).map(|v| v.as_mut_slice())
    }
}