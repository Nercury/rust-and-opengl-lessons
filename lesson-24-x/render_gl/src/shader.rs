use gl;
use na;
use resources::{self, Resource, Resources};
use std;
use std::ffi::{CStr, CString};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to load resource {}", name)]
    ResourceLoad {
        name: String,
        #[cause]
        inner: resources::Error,
    },
    #[fail(
        display = "Can not determine shader type for resource {}",
        name
    )]
    CanNotDetermineShaderTypeForResource { name: String },
    #[fail(display = "Failed to compile shader {}: {}", name, message)]
    CompileError { name: String, message: String },
    #[fail(display = "Failed to link program {}: {}", name, message)]
    LinkError { name: String, message: String },
}

pub struct Program {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Program, Error> {
        const POSSIBLE_EXT: [&str; 2] = [".vert", ".frag"];

        let resource_names = POSSIBLE_EXT
            .iter()
            .map(|file_extension| format!("{}{}", name, file_extension))
            .collect::<Vec<String>>();

        let shaders = resource_names
            .iter()
            .map(|resource_name| Shader::from_res(gl, res, resource_name))
            .collect::<Result<Vec<Shader>, Error>>()?;

        Program::from_shaders(gl, &shaders[..]).map_err(|message| Error::LinkError {
            name: name.into(),
            message,
        })
    }

    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders {
            unsafe {
                gl.AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl.LinkProgram(program_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl.GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                gl.DetachShader(program_id, shader.id());
            }
        }

        Ok(Program {
            gl: gl.clone(),
            id: program_id,
        })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }

    pub fn get_uniform_location(&self, name: &str) -> Option<i32> {
        let cname = CString::new(name).expect("expected uniform name to have no nul bytes");

        let location = unsafe {
            self.gl
                .GetUniformLocation(self.id, cname.as_bytes_with_nul().as_ptr() as *const i8)
        };

        if location == -1 {
            return None;
        }

        Some(location)
    }

    pub fn set_uniform_matrix_4fv(&self, location: i32, value: &na::Matrix4<f32>) {
        unsafe {
            self.gl.UniformMatrix4fv(
                location,
                1,
                gl::FALSE,
                value.as_slice().as_ptr() as *const f32,
            );
        }
    }

    pub fn set_uniform_3f(&self, location: i32, value: &na::Vector3<f32>) {
        unsafe {
            self.gl.Uniform3f(location, value.x, value.y, value.z);
        }
    }

    pub fn set_uniform_1i(&self, location: i32, index: i32) {
        unsafe {
            self.gl.Uniform1i(location, index);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    resource: Resource,
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Shader, Error> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] =
            [(".vert", gl::VERTEX_SHADER), (".frag", gl::FRAGMENT_SHADER)];

        let shader_kind = POSSIBLE_EXT
            .iter()
            .find(|&&(file_extension, _)| name.ends_with(file_extension))
            .map(|&(_, kind)| kind)
            .ok_or_else(|| Error::CanNotDetermineShaderTypeForResource { name: name.into() })?;

        Shader::from_resource(gl, res.resource(name), shader_kind)
    }

    pub fn from_resource(
        gl: &gl::Gl,
        resource: Resource,
        kind: gl::types::GLenum,
    ) -> Result<Shader, Error> {
        let id = shader_from_source(gl, &resource, kind)?;
        Ok(Shader {
            resource,
            gl: gl.clone(),
            id,
        })
    }

    pub fn from_vert_resource(gl: &gl::Gl, resource: Resource) -> Result<Shader, Error> {
        Shader::from_resource(gl, resource, gl::VERTEX_SHADER)
    }

    pub fn from_frag_resource(gl: &gl::Gl, resource: Resource) -> Result<Shader, Error> {
        Shader::from_resource(gl, resource, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn is_modified(&self) -> bool {
        self.resource.is_modified()
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

fn shader_from_source(
    gl: &gl::Gl,
    resource: &Resource,
    kind: gl::types::GLenum,
) -> Result<gl::types::GLuint, Error> {
    let mut source = resource.get().map_err(|e| Error::ResourceLoad {
        name: resource.name(),
        inner: e,
    })?;

    source.push(b'\0');
    let cstr = CStr::from_bytes_with_nul(&source).unwrap();

    let id = unsafe { gl.CreateShader(kind) };
    unsafe {
        gl.ShaderSource(id, 1, &cstr.as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(Error::CompileError {
            name: resource.name(),
            message: error.to_string_lossy().into_owned(),
        });
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
