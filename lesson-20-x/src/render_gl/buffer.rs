use gl;

pub trait BufferType {
    const BUFFER_TYPE: gl::types::GLuint;
}

pub struct BufferTypeArray;
impl BufferType for BufferTypeArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ARRAY_BUFFER;
}

pub struct BufferTypeElementArray;
impl BufferType for BufferTypeElementArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ELEMENT_ARRAY_BUFFER;
}

pub struct Buffer<B>
where
    B: BufferType,
{
    gl: gl::Gl,
    vbo: gl::types::GLuint,
    _marker: ::std::marker::PhantomData<B>,
}

impl<B> Buffer<B>
where
    B: BufferType,
{
    pub fn new(gl: &gl::Gl) -> Buffer<B> {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo);
        }

        Buffer {
            gl: gl.clone(),
            vbo,
            _marker: ::std::marker::PhantomData,
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, self.vbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, 0);
        }
    }

    pub fn static_draw_data<T>(&self, data: &[T]) {
        unsafe {
            self.gl.BufferData(
                B::BUFFER_TYPE, // target
                (data.len() * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );
        }
    }

    pub fn dynamic_draw_data<T>(&self, data: &[T]) {
        unsafe {
            self.gl.BufferData(
                B::BUFFER_TYPE, // target
                (data.len() * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::DYNAMIC_DRAW, // usage
            );
        }
    }

    pub fn dynamic_draw_data_null<T>(&self, size: usize) {
        unsafe {
            self.gl.BufferData(
                B::BUFFER_TYPE, // target
                (size * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // size of data in bytes
                ::std::ptr::null() as *const gl::types::GLvoid, // pointer to data
                gl::DYNAMIC_DRAW, // usage
            );
        }
    }

    pub unsafe fn map_buffer_range_write_invalidate<'r, T>(
        &self,
        offset: usize,
        size: usize,
    ) -> Option<MappedBuffer<'r, B, T>> {
        let ptr = self.gl.MapBufferRange(
            B::BUFFER_TYPE,                                                 // target
            (offset * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr, // offset
            (size * ::std::mem::size_of::<T>()) as gl::types::GLsizeiptr,   //  length
            gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_RANGE_BIT,               // usage
        );
        if ptr == ::std::ptr::null_mut() {
            return None;
        }
        return Some(MappedBuffer {
            gl: self.gl.clone(),
            data: ::std::slice::from_raw_parts_mut(ptr as *mut T, size),
            _marker: ::std::marker::PhantomData,
        });
    }
}

impl<B> Drop for Buffer<B>
where
    B: BufferType,
{
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteBuffers(1, &mut self.vbo);
        }
    }
}

pub struct MappedBuffer<'a, B, DataT: 'a>
where
    B: BufferType,
{
    gl: gl::Gl,
    data: &'a mut [DataT],
    _marker: ::std::marker::PhantomData<B>,
}

impl<'a, B, DataT: 'a> ::std::ops::Deref for MappedBuffer<'a, B, DataT>
where
    B: BufferType,
{
    type Target = [DataT];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, B, DataT: 'a> ::std::ops::DerefMut for MappedBuffer<'a, B, DataT>
where
    B: BufferType,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, B, DataT: 'a> Drop for MappedBuffer<'a, B, DataT>
where
    B: BufferType,
{
    fn drop(&mut self) {
        unsafe {
            self.gl.UnmapBuffer(B::BUFFER_TYPE);
        }
    }
}

pub type ArrayBuffer = Buffer<BufferTypeArray>;
pub type ElementArrayBuffer = Buffer<BufferTypeElementArray>;

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
            vao,
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
