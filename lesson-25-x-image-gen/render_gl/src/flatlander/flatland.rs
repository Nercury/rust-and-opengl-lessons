use metrohash::MetroHashMap;
use slotmap;
use crate::na;
use super::{FlatlanderVertex, FlatlanderGroupDrawData, DrawIndirectCmd, FlatlandItem};

#[derive(Copy, Clone)]
pub struct AlphabetSlotData {
    count: isize,
}

pub struct AlphabetData {
    pub map: MetroHashMap<u32, usize>,
    pub entries: Vec<AlphabetEntry>,
    total_vertices: usize,
    total_indices: usize,
}

impl AlphabetData {
    pub fn new() -> AlphabetData {
        AlphabetData {
            map: MetroHashMap::default(),
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

        let previous_indices = self.total_indices;

        self.total_vertices += vertices.len();
        self.total_indices += indices.len();

        self.entries.push(AlphabetEntry { vertices, indices, previous_indices });
        self.map.insert(id, index);

        index
    }
}

pub struct AlphabetEntry {
    pub vertices: Vec<FlatlanderVertex>,
    pub indices: Vec<u16>,
    pub previous_indices: usize,
}

#[derive(Debug)]
struct AlphabetDataIndexOffset {
    first_index: usize,
}

#[derive(Copy, Clone)]
pub struct GroupSlotData {
}

pub struct GroupData {
    pub transform: na::Projective3<f32>,
    pub color: na::Vector4<u8>,
    pub alphabet_slot: AlphabetSlot,
    pub items: Vec<FlatlandItem>,
}

new_key_type! { pub struct AlphabetSlot; }
new_key_type! { pub struct GroupSlot; }

pub struct Flatland {
    pub alphabet_slots: slotmap::SlotMap<AlphabetSlot, AlphabetSlotData>,
    pub alphabet_data: slotmap::SecondaryMap<AlphabetSlot, AlphabetData>,

    alphabet_data_index_offsets: slotmap::SecondaryMap<AlphabetSlot, AlphabetDataIndexOffset>,
    alphabet_data_index_offsets_invalidated: bool,

    pub group_slots: slotmap::SlotMap<GroupSlot, GroupSlotData>,
    pub group_data: slotmap::SecondaryMap<GroupSlot, GroupData>,

    pub alphabets_invalidated: bool,
    pub groups_invalidated: bool,
    pub draw_invalidated: bool,

    total_alphabet_vertices: usize,
    total_alphabet_indices: usize,
}

impl Flatland {
    pub fn new() -> Flatland {
        Flatland {
            alphabet_slots: slotmap::SlotMap::with_key(),
            alphabet_data: slotmap::SecondaryMap::new(),

            alphabet_data_index_offsets: slotmap::SecondaryMap::new(),
            alphabet_data_index_offsets_invalidated: false,

            group_slots: slotmap::SlotMap::with_key(),
            group_data: slotmap::SecondaryMap::new(),

            alphabets_invalidated: false,
            groups_invalidated: false,
            draw_invalidated: false,

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

    fn ensure_alphabet_data_index_offsets(&mut self) {
        if self.alphabet_data_index_offsets_invalidated {

            self.alphabet_data_index_offsets.clear();
            self.alphabet_data_index_offsets.extend(self.alphabet_data
                .iter()
                .scan(0, |previous_indices, (slot, AlphabetData { ref total_indices, .. })| {
                    let first_index = *previous_indices;
                    *previous_indices += total_indices;
                    Some((slot, first_index))
                })
                .map(|(slot, first_index)|
                    (slot, AlphabetDataIndexOffset {
                        first_index,
                    })
                ));

            self.alphabet_data_index_offsets_invalidated = false;
        }
    }

    pub fn groups_len(&self) -> usize {
        self.group_data.values().map(|g| g.items.len()).sum()
    }

    pub fn groups_draw_data<'r>(&'r mut self) -> impl Iterator<Item = FlatlanderGroupDrawData> + 'r {
        self.ensure_alphabet_data_index_offsets();

        fn unpack<'p>(
            group_data: &'p slotmap::SecondaryMap<GroupSlot, GroupData>,
            alphabet_data: &'p slotmap::SecondaryMap<AlphabetSlot, AlphabetData>,
            alphabet_data_index_offsets: &'p slotmap::SecondaryMap<AlphabetSlot, AlphabetDataIndexOffset>
        ) -> impl Iterator<Item = FlatlanderGroupDrawData> + 'p {
            group_data
                .values()
                .flat_map(move |group| group.items.iter().map(move |i| {
                    let alphabet_slot = group.alphabet_slot;
                    let (previous_indices, num_indices) = alphabet_data[alphabet_slot].entries.get(i.alphabet_entry_index)
                        .map(|e| (e.previous_indices as u32, e.indices.len() as u32))
                        .expect("expected alphabet entry to exist");
                    let first_alphabet_index = alphabet_data_index_offsets[alphabet_slot].first_index as u32;

                    (num_indices, first_alphabet_index + previous_indices, i.x_offset, i.y_offset, group.transform, group.color)
                }))
                .enumerate()
                .map(|(i, (num_indices, first_index, x_offset, y_offset, transform, color))| FlatlanderGroupDrawData {
                    cmd: DrawIndirectCmd {
                        count: num_indices,
                        prim_count: 1,
                        first_index,
                        base_vertex: 0,
                        base_instance: i as u32
                    },
                    x_offset: x_offset as f32,
                    y_offset: y_offset as f32,
                    transform,
                    color
                })
        }

        unpack(&self.group_data, &self.alphabet_data, &self.alphabet_data_index_offsets)
    }

    pub fn create_flatland_group_with_items(&mut self, &transform: &na::Projective3<f32>, color: na::Vector4<u8>, alphabet_slot: AlphabetSlot, items: Vec<FlatlandItem>) -> GroupSlot {
        let slot = self.group_slots.insert(GroupSlotData {});
        self.group_data.insert(slot, GroupData {
            transform,
            alphabet_slot,
            items,
            color,
        });

        self.groups_invalidated = true;
        self.draw_invalidated = true;

        slot
    }

    pub fn update_items<'p>(&mut self, slot: GroupSlot, items: impl Iterator<Item = &'p FlatlandItem>) {
        self.group_data[slot].items.clear();
        self.group_data[slot].items.extend(items);

        self.groups_invalidated = true;
        self.draw_invalidated = true;
    }

    pub fn update_transform(&mut self, slot: GroupSlot, &transform: &na::Projective3<f32>) {
        self.group_data[slot].transform = transform;

        self.draw_invalidated = true;
    }

    pub fn update_color(&mut self, slot: GroupSlot, color: na::Vector4<u8>) {
        self.group_data[slot].color = color;

        self.draw_invalidated = true;
    }

    pub fn delete_flatland_group(&mut self, slot: GroupSlot) {
        self.group_slots.remove(slot);
        self.group_data.remove(slot);

        self.groups_invalidated = true;
        self.draw_invalidated = true;
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
        self.alphabet_data_index_offsets_invalidated = true;
        self.groups_invalidated = true;
        self.draw_invalidated = true;

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
        self.alphabet_data_index_offsets_invalidated = true;
        self.groups_invalidated = true;
        self.draw_invalidated = true;
    }
}
