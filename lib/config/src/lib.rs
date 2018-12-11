use std::rc::Rc;
use std::cell::RefCell;
use toml_edit as toml;

mod shared;

impl ConfigItem for i32 {
    fn serialize(&self, item: &mut toml::Item) {
        *item = toml::value(*self as i64)
    }

    fn deserialize(&mut self, item: &toml::Item) {
        if let Some(v) = item.as_integer() {
            if v <= std::i32::MAX as i64 {
                *self = v as i32;
            }
        }
    }
}

pub trait ConfigItem {
    fn serialize(&self, item: &mut toml::Item);
    fn deserialize(&mut self, item: &toml::Item);
}

pub struct Config {
    shared: Rc<RefCell<shared::InnerConfig>>,
}

impl Config {
    pub fn new(res: resources::Resource) -> Config {
        Config {
            shared: Rc::new(RefCell::new(shared::InnerConfig::new(res)))
        }
    }

    pub fn pick<T>(&self, path: &[&str]) -> Pick<T>
        where T: Default + ConfigItem
    {
        let mut shared = self.shared.borrow_mut();
        let (slot, data) = shared.pick_create(path);

        let value = match data {
            None => T::default(),
            Some(item) => {
                let mut value = T::default();
                value.deserialize(item);
                value
            }
        };

        Pick {
            value,
            slot,
            shared: self.shared.clone(),
        }
    }

    pub fn should_persist(&self) -> bool {
        let shared = self.shared.borrow();
        shared.should_persist()
    }

    pub fn persist(&self) -> Result<(), resources::Error> {
        let mut shared = self.shared.borrow_mut();
        shared.persist()
    }
}

pub struct Pick<T> {
    value: T,
    slot: usize,
    shared: Rc<RefCell<shared::InnerConfig>>,
}

impl<T> Pick<T> where T: ConfigItem {
    pub fn is_modified(&self) -> bool {
        false
    }

    pub fn modify(&mut self, mut fun: impl FnMut(&mut T)) {
        fun(&mut self.value);
        let mut shared = self.shared.borrow_mut();
        if let Some(item) = shared.pick_mut(self.slot) {
            self.value.serialize(item)
        }
    }
}

impl<T> std::ops::Deref for Pick<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}