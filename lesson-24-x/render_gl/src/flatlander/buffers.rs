use data;
use buffer::Buffer;
use buffer::VertexArray;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FlatlanderVertex {
    #[location = "0"]
    pub pos: data::f32_f32,
    #[location = "1"]
    pub normal: data::f32_f32,
}

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FlatlanderVertexDrawId {
    #[location = "2"]
    #[divisor = "1"]
    pub x_offset: data::f32_,
    #[location = "3"]
    #[divisor = "1"]
    pub y_offset: data::f32_,
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct DrawIndirectCmd {
    pub count: u32,
    pub prim_count: u32,
    pub first_index: u32,
    pub base_vertex: u32,
    pub base_instance: u32,
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FlatlanderGroupDrawData {
    pub cmd: DrawIndirectCmd,
    pub x_offset: f32,
    pub y_offset: f32,
}

pub struct Buffers {
    vertices: Storage,
    indices: Storage,
    draw_id: Storage,
    pub indirect: Storage,

    pub groups_simple: Vec<FlatlanderGroupDrawData>,

    pub lines_vao: VertexArray,
}

pub struct Storage {
    pub buffer: Buffer,
    pub len: usize,
}

impl Storage {
    pub fn new(buffer: Buffer, len: usize) -> Storage {
        Storage {
            buffer,
            len,
        }
    }

    pub fn upload<T, I: Iterator<Item = T>>(&mut self, items_len: usize, items: I) {
        if items_len > 0 {
            let should_recreate_buffer = self.len < items_len;

            self.buffer.bind();

            if should_recreate_buffer {
                trace!("stream_draw_data_null {}", items_len);
                self.buffer.stream_draw_data_null::<T>(items_len);
            }

            if let Some(mut buffer) = unsafe {
                self.buffer
                    .map_buffer_range_write_invalidate::<T>(0, items_len)
            } {
                trace!("write buffer");
                for (index, item) in items.enumerate().take(items_len) {
                    *unsafe { buffer.get_unchecked_mut(index) } = item;
                }
            }

            self.buffer.unbind();
        }

        self.len = items_len;
    }
}

impl Buffers {
    pub fn new(gl: &gl::Gl) -> Buffers {
        let vertices = Buffer::new_array(&gl);
        let indices = Buffer::new_element_array(&gl);
        let indirect = Buffer::new_draw_indirect(&gl);
        let draw_id = Buffer::new_array(&gl);

        let lines_vao = VertexArray::new(gl);

        lines_vao.bind();

        vertices.bind();
        FlatlanderVertex::vertex_attrib_pointers(gl);
        draw_id.bind();
        FlatlanderVertexDrawId::vertex_attrib_pointers(gl);
        draw_id.unbind();

        indices.bind();
        lines_vao.unbind();

        indices.unbind();

        Buffers {
            vertices: Storage::new(vertices, 0),
            indices: Storage::new(indices, 0),
            indirect: Storage::new(indirect, 0),
            draw_id: Storage::new(draw_id, 0),
            lines_vao,
            groups_simple: Vec::new(),
        }
    }


    pub fn upload_vertices(&mut self, items_len: usize, items: impl Iterator<Item = FlatlanderVertex>) {
        self.vertices.upload(items_len, items);
    }

    pub fn upload_indices(&mut self, items_len: usize, items: impl Iterator<Item = u16>) {
        self.indices.upload(items_len, items);
    }

    pub fn upload_groups(&mut self, items_len: usize, items: impl Iterator<Item = FlatlanderGroupDrawData>) {
        let items = items.collect::<Vec<_>>();

        self.draw_id.upload(items_len, items.iter()
            .map(|i| FlatlanderVertexDrawId {
                x_offset: i.x_offset.into(),
                y_offset: i.y_offset.into()
            }));

        self.indirect.upload(items_len, items.into_iter()
            .map(|i| i.cmd));
    }
}