#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

decl_derive!([VertexAttribPointers, attributes()] => vertex_attrib_pointers_derive);

fn vertex_attrib_pointers_derive(s: synstructure::Structure) -> quote::Tokens {
    quote! {
        impl Vertex {
            fn vertex_attrib_pointers(gl: &gl::Gl) {
                let stride = std::mem::size_of::<Self>(); // byte offset between consecutive attributes

                let location = 0; // layout (location = 0)
                let offset = 0; // offset of the first component

                unsafe {
                    data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
                }

                let location = 1; // layout (location = 1)
                let offset = offset + std::mem::size_of::<data::f32_f32_f32>(); // offset of the first component

                unsafe {
                    data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
                }
            }
        }
    }
}