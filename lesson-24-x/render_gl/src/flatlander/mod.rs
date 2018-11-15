use gl;
use na;
use failure;
use resources::Resources;
use ColorBuffer;
use Program;
use std::rc::Rc;
use std::cell::RefCell;

mod buffers;
mod flatland;

pub use self::buffers::{FlatlanderVertex, FlatlanderGroupDrawData, DrawIndirectCmd};

pub struct Flatlander {
    program: Program,
    program_view_projection_location: Option<i32>,
    program_model_matrix_location: Option<i32>,
    program_color_location: Option<i32>,
    flatland: Rc<RefCell<flatland::Flatland>>,
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
            flatland: Rc::new(RefCell::new(flatland::Flatland::new())),
            buffers: None,
            draw_enabled: true,
        })
    }

    pub fn toggle(&mut self) {
        self.draw_enabled = !self.draw_enabled;
    }

    fn check_if_invalidated_and_reinitialize(&mut self, gl: &gl::Gl) {
        let mut flatland = self.flatland.borrow_mut();

        if flatland.alphabets_invalidated {
            if self.buffers.is_none() {
                self.buffers = Some(buffers::Buffers::new(gl));
            }

            if let Some(ref mut buffers) = self.buffers {
                buffers.upload_vertices(flatland.alphabet_vertices_len(), flatland.alphabet_vertices());
                buffers.upload_indices(flatland.alphabet_indices_len(), flatland.alphabet_indices());
                buffers.upload_groups(flatland.groups_len(), flatland.groups_draw_data());
            }

            flatland.alphabets_invalidated = false;
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

            if let Some(ref buffers) = self.buffers {
                self.program.set_used();
                if let Some(loc) = self.program_view_projection_location {
                    self.program.set_uniform_matrix_4fv(loc, &vp_matrix);
                }

                let program_model_matrix_location = self
                    .program_model_matrix_location
                    .expect("Flatland Model uniform must exist");

                let program_color_location = self
                    .program_color_location
                    .expect("Flatland Color uniform must exist");

                self.program.set_uniform_matrix_4fv(program_model_matrix_location,
                                                    &(
                                                        na::Matrix4::<f32>::new_translation(&na::Vector3::new(100.0, 200.0, 0.0)) *
                                                        na::Matrix4::<f32>::new_scaling(0.05) *
                                                        na::Matrix4::<f32>::new_nonuniform_scaling(&na::Vector3::new(1.0, -1.0, 1.0))
                                                    )
                );
                self.program.set_uniform_4f(program_color_location, &na::Vector4::<f32>::new(1.0, 1.0, 1.0, 1.0));

                buffers.lines_vao.bind();
                buffers.indirect.buffer.bind();

                unsafe {
                    target.set_default_blend_func(gl);
                    target.enable_blend(gl);
                    target.front_face_cw(gl);

                    if gl.MultiDrawElementsIndirect.is_loaded() {
                        // open gl 4.3
                        gl.MultiDrawElementsIndirect(
                            gl::TRIANGLES,
                            gl::UNSIGNED_SHORT,
                            0 as *const ::std::ffi::c_void,
                            buffers.indirect.len as i32,
                            ::std::mem::size_of::<DrawIndirectCmd>() as i32
                        );
                    } else {
                        // open gl 4.1
                        // manual implementation of MultiDrawElementsIndirect

                        for i in 0..buffers.indirect.len {
                            gl.DrawElementsIndirect(
                                gl::TRIANGLES,
                                gl::UNSIGNED_SHORT,
                                (i as u32 * ::std::mem::size_of::<DrawIndirectCmd>() as u32) as *const ::std::ffi::c_void
                            );
                        }
                    }

                    target.front_face_ccw(gl);
                    target.disable_blend(gl);
                }

                buffers.indirect.buffer.unbind();
                buffers.lines_vao.unbind();
            }
        }
    }
}

pub struct Alphabet {
    slot: flatland::AlphabetSlot,
    flatland: Rc<RefCell<flatland::Flatland>>,
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

pub struct FlatlandItem {
    pub alphabet_entry_index: usize,
    pub x_offset: i32,
    pub y_offset: i32,
}

pub struct FlatlandGroup {
    alphabet: Alphabet,
    group_slot: flatland::GroupSlot,
}

impl FlatlandGroup {
    pub fn new(alphabet: Alphabet, items: Vec<FlatlandItem>) -> FlatlandGroup {
        let id = alphabet.flatland.borrow_mut().create_flatland_group_with_items(alphabet.slot, items);

        FlatlandGroup {
            alphabet: alphabet.clone(),
            group_slot: id,
        }
    }
}

impl Drop for FlatlandGroup {
    fn drop(&mut self) {
        self.alphabet.flatland.borrow_mut().delete_flatland_group(self.group_slot);
    }
}