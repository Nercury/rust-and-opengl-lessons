mod paths;

use slab::Slab;
use resources::Resource;
use toml_edit as toml;
use log::*;

struct PickData {
    path: paths::Path,
    should_deserialize: bool,
}

struct Contents {
    doc: Option<toml::Document>,
}

impl Contents {
    pub fn new() -> Contents {
        Contents {
            doc: None,
        }
    }

    fn load_if_not_loaded(&mut self, res: &resources::Resource) {
        if self.doc.is_none() {
            self.reload(res);
        }
    }

    fn reload(&mut self, res: &resources::Resource) {
        self.doc = res.get().map_err(|e| error!("resource error: {:?}", e))
            .ok()
            .and_then(|bytes|
                String::from_utf8(bytes)
                    .map_err(|e| error!("resource did not contain valid unicode: {:?}", e))
                    .ok()
            )
            .and_then(|string|
                string.parse::<toml::Document>()
                    .map_err(|e| error!("failed to parse toml: {:?}", e))
                    .ok()
            );
    }

    fn item_at<'a, 'e>(&'a mut self, res: &resources::Resource, path: impl Iterator<Item = &'e str>) -> Option<&'a toml::Item> {
        self.load_if_not_loaded(res);
        self.doc
            .as_ref()
            .and_then(move |doc| table_path_to_item(doc.as_table(), path))
    }

    fn item_at_mut<'a, 'e>(&'a mut self, res: &resources::Resource, path: impl Iterator<Item = &'e str>) -> Option<&'a mut toml::Item> {
        self.load_if_not_loaded(res);
        self.doc
            .as_mut()
            .and_then(move |doc| table_path_to_item_mut(doc.as_table_mut(), path))
    }
}

pub struct InnerConfig {
    picks: Slab<PickData>,

    path_storage: paths::Storage,

    res: resources::Resource,
    contents: Contents,

    has_new_changes_to_persist: bool,
    has_not_reloaded_picks: bool,
}

impl InnerConfig {
    pub fn new(res: Resource) -> InnerConfig {
        InnerConfig {
            picks: Slab::new(),
            path_storage: paths::Storage::new(),
            res,
            contents: Contents::new(),
            has_new_changes_to_persist: false,
            has_not_reloaded_picks: false,
        }
    }

    pub fn should_persist(&self) -> bool {
        self.has_new_changes_to_persist
    }

    pub fn persist(&mut self) -> Result<(), resources::Error> {
        let data = self.contents.doc.as_ref().map(|doc| doc.to_string());
        if let Some(data) = data {
            self.res.write(data.as_bytes())?;
        }

        self.has_new_changes_to_persist = false;

        Ok(())
    }

    pub fn pick_create<'a, 'e>(&'a mut self, path: &'e [&'e str]) -> (usize, Option<&'a toml::Item>) {
        let pick_slot = self.picks.insert(PickData {
            path: paths::Path::new(&mut self.path_storage, path),
            should_deserialize: false,
        });

        (pick_slot, self.contents.item_at(&self.res, path.iter().map(|i| *i)))
    }

    pub fn pick_mut(&mut self, slot: usize) -> Option<&mut toml::Item> {
        self.has_new_changes_to_persist = true;
        self.should_reload_others_except(slot);

        let path = self.picks.get(slot).map(|p| &p.path)?;
        let iter = path.components(&self.path_storage);
        self.contents.item_at_mut(&self.res, iter)
    }

    fn should_reload_others_except(&mut self, slot: usize) {
        let mut has_not_reloaded_picks = false;
        for (other_slot, pick_data) in self.picks.iter_mut() {
            if other_slot != slot {
                pick_data.should_deserialize = true;
                has_not_reloaded_picks = true;
            }
        }

        if has_not_reloaded_picks {
            self.has_not_reloaded_picks = true;
        }
    }
}

fn table_path_to_item<'a, 'e>(table: &'a toml::Table, mut iter: impl Iterator<Item = &'e str>) -> Option<&'a toml::Item> {
    let next = iter.next();
    match next {
        Some(key) => table.get(key)
            .and_then(|item| item_path_to_item(item, iter)),
        None => None,
    }
}

fn table_path_to_item_mut<'a, 'e>(table: &'a mut toml::Table, mut iter: impl Iterator<Item = &'e str>) -> Option<&'a mut toml::Item> {
    let next = iter.next();
    match next {
        Some(key) => Some(item_path_to_item_mut(table.entry(key), iter)),
        None => None,
    }
}

fn item_path_to_item<'a, 'e>(item: &'a toml::Item, mut iter: impl Iterator<Item = &'e str>) -> Option<&'a toml::Item> {
    let next = iter.next();
    match next {
        Some(key) => match item {
            toml::Item::Table(t) => t.get(key).and_then(|item| item_path_to_item(item, iter)),
            _ => None,
        },
        None => Some(item),
    }
}

fn item_path_to_item_mut<'a, 'e>(item: &'a mut toml::Item, mut iter: impl Iterator<Item = &'e str>) -> &'a mut toml::Item {
    let next = iter.next();
    match next {
        Some(key) => {
            if !item.is_table() {
                *item = toml::Item::Table(toml::Table::new());
            }
            item_path_to_item_mut(item.as_table_mut().unwrap().entry(key), iter)
        },
        None => item,
    }
}