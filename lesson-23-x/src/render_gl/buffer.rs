use gl;

//pub struct BufferData<T> {
//    buffer: Buffer,
//    _marker: ::std::marker::PhantomData<T>,
//}

pub struct Buffer {
    gl: gl::Gl,
    buffer_type: gl::types::GLuint,
    vbo: gl::types::GLuint,
}

impl Buffer where {
    pub fn new_array(gl: &gl::Gl) -> Buffer {
        Self::new(gl, gl::ARRAY_BUFFER)
    }

    pub fn new_element_array(gl: &gl::Gl) -> Buffer {
        Self::new(gl, gl::ELEMENT_ARRAY_BUFFER)
    }

    pub fn new(gl: &gl::Gl, buffer_type: gl::types::GLuint) -> Buffer {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }

        Buffer {
            gl: gl.clone(),
            buffer_type,
            vbo,
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindBuffer(self.buffer_type, self.vbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindBuffer(self.buffer_type, 0);
        }
    }

    pub fn static_draw_data<T>(&self, data: &[T]) {
        unsafe {
            self.gl.BufferData(
                self.buffer_type, // target
                (data.len() * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );
        }
    }

    pub fn stream_draw_data<T>(&self, data: &[T]) {
        unsafe {
            self.gl.BufferData(
                self.buffer_type, // target
                (data.len() * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STREAM_DRAW, // usage
            );
        }
    }

    pub fn stream_draw_data_null<T>(&self, size: usize) {
        unsafe {
            self.gl.BufferData(
                self.buffer_type, // target
                (size * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                ::std::ptr::null() as *const gl::types::GLvoid, // pointer to data
                gl::STREAM_DRAW, // usage
            );
        }
    }

    pub unsafe fn map_buffer_range_write_invalidate<'r, T>(&self, offset: usize, size: usize) -> Option<MappedBuffer<'r, T>> {
        let ptr = self.gl.MapBufferRange(
            self.buffer_type, // target
            (offset * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // offset
            (size * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, //  length
            gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_RANGE_BIT, // usage
        );
        if ptr == ::std::ptr::null_mut() {
            return None;
        }
        return Some(MappedBuffer {
            gl: self.gl.clone(),
            buffer_type: self.buffer_type,
            data: ::std::slice::from_raw_parts_mut(ptr as *mut T, size),
            position: 0,
        });
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteBuffers(1, &mut self.vbo);
        }
    }
}

pub struct MappedBuffer<'a, DataT: 'a> {
    gl: gl::Gl,
    buffer_type: gl::types::GLuint,
    data: &'a mut [DataT],
    position: usize,
}

impl<'a, DataT: 'a> MappedBuffer<'a, DataT> {
    pub fn clear(&mut self) {
        self.position = 0;
    }

    pub fn push(&mut self, data: DataT) {
        if self.position < self.data.len() {
            *unsafe { self.data.get_unchecked_mut(self.position) } = data;
            self.position += 1;
        }
    }
}

impl<'a, DataT: 'a> ::std::ops::Deref for MappedBuffer<'a, DataT> {
    type Target = [DataT];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, DataT: 'a> ::std::ops::DerefMut for MappedBuffer<'a, DataT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, DataT: 'a> Drop for MappedBuffer<'a, DataT> {
    fn drop(&mut self) {
        unsafe {
            self.gl.UnmapBuffer(self.buffer_type);
        }
    }
}

pub struct VertexArray {
    gl: gl::Gl,
    vao: gl::types::GLuint,
}

impl VertexArray {
    pub fn new(gl: &gl::Gl) -> VertexArray {
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut vao);
        }

        VertexArray {
            gl: gl.clone(),
            vao
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindVertexArray(self.vao);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindVertexArray(0);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteVertexArrays(1, &mut self.vao);
        }
    }
}