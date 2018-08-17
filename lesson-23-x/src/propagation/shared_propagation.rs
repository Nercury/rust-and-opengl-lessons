use slab::Slab;

pub struct SharedPropagation {
    slots: Slab<bool>, // can this be bit-packed? then we could zero evrything out with a linear scan
}

impl SharedPropagation {
    pub fn new() -> SharedPropagation {
        SharedPropagation {
            slots: Slab::new(),
        }
    }

    pub fn clear(&mut self) {
        for (_, slot) in &mut self.slots {
            *slot = false;
        }
    }

    pub fn mark_modified(&mut self, key: usize) {
        if let Some(slot) = self.slots.get_mut(key) {
            *slot = true;
        }
    }

    pub fn is_modified(&self, key: usize) -> bool {
        if let Some(slot) = self.slots.get(key) {
            return *slot;
        }
        false
    }
}
