use gl;
use na;
use failure;
use resources::Resources;
use ColorBuffer;
use Program;
use std::rc::Rc;
use std::cell::RefCell;

mod buffers;

pub use self::buffers::FlatlanderVertex;

pub struct Flatlander {
    program: Program,
    program_view_projection_location: Option<i32>,
    program_model_matrix_location: Option<i32>,
    program_color_location: Option<i32>,
    flatland: Rc<RefCell<shared::Flatland>>,
    buffers: Option<buffers::Buffers>,
    draw_enabled: bool,
}

impl Flatlander {
    pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Flatlander, failure::Error> {
        let program = Program::from_res(gl, res, "shaders/render_gl/flatland")?;
        let program_view_projection_location = program.get_uniform_location("ViewProjection");
        let program_model_matrix_location = program.get_uniform_location("Model");
        let program_color_location = program.get_uniform_location("Color");

        Ok(Flatlander {
            program,
            program_view_projection_location,
            program_model_matrix_location,
            program_color_location,
            flatland: Rc::new(RefCell::new(shared::Flatland::new())),
            buffers: None,
            draw_enabled: true,
        })
    }

    pub fn toggle(&mut self) {
        self.draw_enabled = !self.draw_enabled;
    }

    fn check_if_invalidated_and_reinitialize(&mut self, gl: &gl::Gl) {
        let mut flatland = self.flatland.borrow_mut();

        if flatland.invalidated {
            if self.buffers.is_none() {
                self.buffers = Some(buffers::Buffers::new(gl));
            }

            if let Some(ref mut buffers) = self.buffers {

            }

            flatland.invalidated = false;
        }
    }

    pub fn create_alphabet(&self) -> Alphabet {
        let mut flatland = self.flatland.borrow_mut();
        let slot = flatland.create_alphabet();
        Alphabet {
            slot,
            flatland: self.flatland.clone(),
        }
    }

    pub fn render(&mut self, gl: &gl::Gl, target: &ColorBuffer, vp_matrix: &na::Matrix4<f32>) {
        if self.draw_enabled {
            self.check_if_invalidated_and_reinitialize(gl);

            if let Some(ref buffers) = self.buffers {}
        }
    }
}

pub struct Alphabet {
    slot: shared::AlphabetSlot,
    flatland: Rc<RefCell<shared::Flatland>>,
}

impl Clone for Alphabet {
    fn clone(&self) -> Self {
        let mut flatland = self.flatland.borrow_mut();
        flatland.inc_alphabet(self.slot);
        Alphabet {
            slot: self.slot,
            flatland: self.flatland.clone(),
        }
    }
}

impl Alphabet {
    pub fn get_entry_index(&self, id: u32) -> Option<usize> {
        let mut flatland = self.flatland.borrow();
        flatland.get_alphabet_entry_index(self.slot, id)
    }

    pub fn add_entry(&self, id: u32, vertices: Vec<FlatlanderVertex>, indices: Vec<u16>) -> usize {
        let mut flatland = self.flatland.borrow_mut();
        flatland.add_alphabet_entry(self.slot, id, vertices, indices)
    }
}

impl Drop for Alphabet {
    fn drop(&mut self) {
        let mut flatland = self.flatland.borrow_mut();
        flatland.dec_alphabet(self.slot);
    }
}

mod shared {
    use int_hash::IntHashMap;
    use slotmap;
    use super::FlatlanderVertex;

    #[derive(Copy, Clone)]
    pub struct AlphabetSlotData {
        count: isize,
    }

    pub struct AlphabetData {
        pub map: IntHashMap<u32, usize>,
        pub entries: Vec<AlphabetEntry>,
    }

    impl AlphabetData {
        pub fn new() -> AlphabetData {
            AlphabetData {
                map: IntHashMap::default(),
                entries: Vec::with_capacity(4096),
            }
        }

        pub fn get_index(&self, id: u32) -> Option<usize> {
            self.map.get(&id).map(|v| *v)
        }

        pub fn add(&mut self, id: u32, vertices: Vec<FlatlanderVertex>, indices: Vec<u16>) -> usize {
            let index = self.entries.len();
            self.entries.push(AlphabetEntry { vertices, indices });
            self.map.insert(id, index);
            index
        }
    }

    pub struct AlphabetEntry {
        pub vertices: Vec<FlatlanderVertex>,
        pub indices: Vec<u16>,
    }

    new_key_type! { pub struct AlphabetSlot; }

    pub struct Flatland {
        pub alphabet_slots: slotmap::HopSlotMap<AlphabetSlot, AlphabetSlotData>,
        pub alphabet_data: slotmap::SparseSecondaryMap<AlphabetSlot, AlphabetData>,

        pub invalidated: bool,
    }

    impl Flatland {
        pub fn new() -> Flatland {
            Flatland {
                alphabet_slots: slotmap::HopSlotMap::with_key(),
                alphabet_data: slotmap::SparseSecondaryMap::new(),

                invalidated: true,
            }
        }

        pub fn create_alphabet(&mut self) -> AlphabetSlot {
            let slot = self.alphabet_slots.insert(AlphabetSlotData { count: 1 });
            self.alphabet_data.insert(slot, AlphabetData::new());
            self.invalidated = true;
            slot
        }

        pub fn get_alphabet_entry_index(&self, slot: AlphabetSlot, id: u32) -> Option<usize> {
            self.alphabet_data[slot].get_index(id)
        }

        pub fn add_alphabet_entry(&mut self, slot: AlphabetSlot, id: u32, vertices: Vec<FlatlanderVertex>, indices: Vec<u16>) -> usize {
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
            self.alphabet_data.remove(slot);
            self.invalidated = true;
        }
    }
}