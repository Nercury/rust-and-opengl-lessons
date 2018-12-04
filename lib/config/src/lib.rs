use std::rc::Rc;
use std::cell::RefCell;

pub struct Config {
    shared: Rc<RefCell<shared::InnerConfig>>,
}

impl Config {
    pub fn new(res: resources::Resource) -> Config {
        Config {
            shared: Rc::new(RefCell::new(shared::InnerConfig::new(res)))
        }
    }

    pub fn pick<T>(&self, name: &str) -> Pick<T>
        where T: Default
    {
        let (index, mut data) = self.shared.borrow_mut().pick(name)
            .unwrap_or_else(|| panic!("config section {:?} is already in use", name));

        let value = if data.is_none() {
            T::default()
        } else {
            unimplemented!("parse value from data")
        };

        Pick {
            value,
            shared: self.shared.clone(),
        }
    }
}

pub struct Pick<T> {
    value: T,
    shared: Rc<RefCell<shared::InnerConfig>>,
}

mod shared {
    use slab::Slab;
    use resources::Resource;
    use metrohash::MetroHashMap;

    struct SlabData {
    }

    pub struct InnerConfig {
        sections: Slab<SlabData>,
        section_name_index: MetroHashMap<String, usize>,
        res: resources::Resource,
    }

    impl InnerConfig {
        pub fn new(res: Resource) -> InnerConfig {
            InnerConfig {
                sections: Slab::new(),
                section_name_index: MetroHashMap::default(),
                res
            }
        }

        pub fn pick(&mut self, section_name: &str) -> Option<(usize, Option<String>)> {
            let existing_section = self.section_name_index.get(section_name).map(|v| *v);

            match existing_section {
                Some(_) => None,
                None => {
                    let index = self.sections.insert(SlabData {});
                    self.section_name_index.insert(section_name.to_string(), index);

                    Some((index, None)) // TODO: load and return section config
                }
            }
        }
    }
}