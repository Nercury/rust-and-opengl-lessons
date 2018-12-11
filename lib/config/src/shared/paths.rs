use metrohash::MetroHashMap;
use slab::Slab;

const MAX_PATH_LEN: usize = 32;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct NameSlot(usize);

pub struct Storage {
    path_names_index: Slab<String>,
    path_names: MetroHashMap<String, NameSlot>,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            path_names_index: Slab::new(),
            path_names: MetroHashMap::default(),
        }
    }

    pub fn name_slot(&mut self, component: &str) -> NameSlot {
        match self.path_names.get(component).map(|v| *v) {
            None => {
                let index = self.path_names_index.insert(component.to_string());
                self.path_names.insert(component.to_string(), NameSlot(index));
                NameSlot(index)
            },
            Some(s) => s,
        }
    }
}

pub struct Path {
    path: [NameSlot; MAX_PATH_LEN],
    path_len: usize,
}

impl Path {
    pub fn new(storage: &mut Storage, str_path: &[&str]) -> Path {
        if str_path.len() > MAX_PATH_LEN {
            unimplemented!("config path length exceeded");
        }

        let mut output_path = [NameSlot(0); MAX_PATH_LEN];
        for (&component, output_path_component) in str_path.iter().zip(output_path.iter_mut()) {
            *output_path_component = storage.name_slot(component);
        }

        Path {
            path: output_path,
            path_len: str_path.len(),
        }
    }

    pub fn components<'r>(&'r self, storage: &'r Storage) -> impl Iterator<Item = &'r str> + 'r {
        self.path.iter()
            .take(self.path_len)
            .map(move |slot| storage.path_names_index[slot.0].as_str())
    }
}
