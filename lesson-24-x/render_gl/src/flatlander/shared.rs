use int_hash::IntHashMap;
use slotmap;
use super::{FlatlanderVertex, FlatlandItem};

#[derive(Copy, Clone)]
pub struct AlphabetSlotData {
    count: isize,
}

pub struct AlphabetData {
    pub map: IntHashMap<u32, usize>,
    pub entries: Vec<AlphabetEntry>,
    total_vertices: usize,
    total_indices: usize,
}

impl AlphabetData {
    pub fn new() -> AlphabetData {
        AlphabetData {
            map: IntHashMap::default(),
            entries: Vec::with_capacity(4096),
            total_vertices: 0,
            total_indices: 0,
        }
    }

    pub fn get_index(&self, id: u32) -> Option<usize> {
        self.map.get(&id).map(|v| *v)
    }

    pub fn add(&mut self, id: u32, vertices: Vec<FlatlanderVertex>, indices: Vec<u16>) -> usize {
        let index = self.entries.len();

        self.total_vertices += vertices.len();
        self.total_indices += indices.len();

        self.entries.push(AlphabetEntry { vertices, indices });
        self.map.insert(id, index);

        index
    }
}

pub struct AlphabetEntry {
    pub vertices: Vec<FlatlanderVertex>,
    pub indices: Vec<u16>,
}

#[derive(Copy, Clone)]
pub struct GroupSlotData {
}

pub struct GroupData {
    pub items: Vec<FlatlandItem>,
}

new_key_type! { pub struct AlphabetSlot; }
new_key_type! { pub struct GroupSlot; }

pub struct Flatland {
    pub alphabet_slots: slotmap::HopSlotMap<AlphabetSlot, AlphabetSlotData>,
    pub alphabet_data: slotmap::SparseSecondaryMap<AlphabetSlot, AlphabetData>,

    pub group_slots: slotmap::HopSlotMap<GroupSlot, GroupSlotData>,
    pub group_data: slotmap::SparseSecondaryMap<GroupSlot, GroupData>,

    pub alphabets_invalidated: bool,
    pub groups_invalidated: bool,

    total_alphabet_vertices: usize,
    total_alphabet_indices: usize,
}

impl Flatland {
    pub fn new() -> Flatland {
        Flatland {
            alphabet_slots: slotmap::HopSlotMap::with_key(),
            alphabet_data: slotmap::SparseSecondaryMap::new(),

            group_slots: slotmap::HopSlotMap::with_key(),
            group_data: slotmap::SparseSecondaryMap::new(),

            alphabets_invalidated: false,
            groups_invalidated: false,

            total_alphabet_vertices: 0,
            total_alphabet_indices: 0,
        }
    }

    pub fn alphabet_vertices_len(&self) -> usize {
        self.total_alphabet_vertices
    }

    pub fn alphabet_vertices<'r>(&'r self) -> impl Iterator<Item = FlatlanderVertex> + 'r {
        self.alphabet_data
            .values()
            .flat_map(|data|
                data.entries.iter()
            )
            .flat_map(|entry|
                entry.vertices.iter().map(|v| *v)
            )
    }

    pub fn alphabet_indices_len(&self) -> usize {
        self.total_alphabet_indices
    }

    pub fn alphabet_indices<'r>(&'r self) -> impl Iterator<Item = u16> + 'r {
        self.alphabet_data
            .values()
            .flat_map(|data|
                data.entries.iter()
            )
            .scan(0, |base_index, AlphabetEntry { ref vertices, ref indices, .. }| {
                let previous_base_index = *base_index;
                *base_index += vertices.len() as u16;
                Some((previous_base_index, indices))
            })
            .flat_map(|(base_index, indices)|
                indices.iter().map(move |index| *index + base_index)
            )
    }

    pub fn create_flatland_group_with_items(&mut self, items: Vec<FlatlandItem>) -> GroupSlot {
        let slot = self.group_slots.insert(GroupSlotData {});
        self.group_data.insert(slot, GroupData {
            items,
        });

        self.groups_invalidated = true;

        slot
    }

    pub fn delete_flatland_group(&mut self, slot: GroupSlot) {
        self.group_slots.remove(slot);
        self.group_data.remove(slot);

        self.groups_invalidated = true;
    }

    pub fn create_alphabet(&mut self) -> AlphabetSlot {
        let slot = self.alphabet_slots.insert(AlphabetSlotData { count: 1 });
        self.alphabet_data.insert(slot, AlphabetData::new());
        slot
    }

    pub fn get_alphabet_entry_index(&self, slot: AlphabetSlot, id: u32) -> Option<usize> {
        self.alphabet_data[slot].get_index(id)
    }

    pub fn add_alphabet_entry(&mut self, slot: AlphabetSlot, id: u32, vertices: Vec<FlatlanderVertex>, indices: Vec<u16>) -> usize {
        self.alphabets_invalidated = true;
        self.groups_invalidated = true;

        self.total_alphabet_vertices += vertices.len();
        self.total_alphabet_indices += indices.len();
        self.alphabet_data[slot].add(id, vertices, indices)
    }

    pub fn inc_alphabet(&mut self, slot: AlphabetSlot) {
        self.alphabet_slots[slot].count += 1;
    }

    pub fn dec_alphabet(&mut self, slot: AlphabetSlot) {
        self.alphabet_slots[slot].count -= 1;

        if self.alphabet_slots[slot].count <= 0 {
            self.delete_alphabet(slot);
        }
    }

    pub fn delete_alphabet(&mut self, slot: AlphabetSlot) {
        self.alphabet_slots.remove(slot);
        let data = self.alphabet_data.remove(slot).expect("expected to remove data when removing the alphabet");
        self.total_alphabet_vertices -= data.total_vertices;
        self.total_alphabet_indices -= data.total_indices;

        self.alphabets_invalidated = true;
        self.groups_invalidated = true;
    }
}
