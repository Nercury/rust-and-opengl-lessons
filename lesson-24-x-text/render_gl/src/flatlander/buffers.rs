use na;
use data;
use buffer::Buffer;
use buffer::VertexArray;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FlatlanderVertex {
    #[location = "0"]
    pub pos: data::f16_f16,
    #[location = "1"]
    pub normal: data::f16_f16,
}

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct FlatlanderVertexDrawId {
    #[location = "2"]
    #[divisor = "1"]
    pub x_offset: data::f16_,
    #[location = "3"]
    #[divisor = "1"]
    pub y_offset: data::f16_,
    #[location = "4"]
    #[divisor = "1"]
    pub model_col0: data::f16_f16_f16_f16,
    #[location = "5"]
    #[divisor = "1"]
    pub model_col1: data::f16_f16_f16_f16,
    #[location = "6"]
    #[divisor = "1"]
    pub model_col2: data::f16_f16_f16_f16,
    #[location = "7"]
    #[divisor = "1"]
    pub model_col3: data::f16_f16_f16_f16,
    #[location = "8"]
    #[divisor = "1"]
    pub color: data::u8_u8_u8_u8_float,
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
pub struct FlatlanderGroupDrawData {
    pub cmd: DrawIndirectCmd,
    pub x_offset: f32,
    pub y_offset: f32,
    pub transform: na::Projective3<f32>,
    pub color: na::Vector4<u8>,
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
                self.buffer.stream_draw_data_null::<T>(items_len);
            }

            if let Some(mut buffer) = unsafe {
                self.buffer
                    .map_buffer_range_write_invalidate::<T>(0, items_len)
            } {
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
        self.draw_id.upload(items_len, items
            .map(|i| {
                let mat: na::Matrix4<f32> = na::convert::<_, na::Matrix4<f32>>(i.transform) *
                    na::Matrix4::<f32>::new_nonuniform_scaling(&na::Vector3::new(1.0, -1.0, 1.0));
                let col0 = mat.column(0);
                let col1 = mat.column(1);
                let col2 = mat.column(2);
                let col3 = mat.column(3);

                FlatlanderVertexDrawId {
                    x_offset: i.x_offset.into(),
                    y_offset: i.y_offset.into(),
                    model_col0: data::f16_f16_f16_f16::from((col0[0], col0[1], col0[2], col0[3])),
                    model_col1: data::f16_f16_f16_f16::from((col1[0], col1[1], col1[2], col1[3])),
                    model_col2: data::f16_f16_f16_f16::from((col2[0], col2[1], col2[2], col2[3])),
                    model_col3: data::f16_f16_f16_f16::from((col3[0], col3[1], col3[2], col3[3])),
                    color: (i.color.x, i.color.y, i.color.z, i.color.w).into(),
                }
            }));
    }

    pub fn upload_draw_commands(&mut self, items_len: usize, items: impl Iterator<Item = FlatlanderGroupDrawData>) {
        self.indirect.upload(items_len, items
            .map(|i| i.cmd));
    }
}