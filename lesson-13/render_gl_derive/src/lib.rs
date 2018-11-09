#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(VertexAttribPointers, attributes(location))]
pub fn vertex_attrib_pointers_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = generate_impl(&ast);
    gen.parse().unwrap()
}

fn generate_impl(ast: &syn::DeriveInput) -> quote::Tokens {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let fields_vertex_attrib_pointer = generate_vertex_attrib_pointer_calls(&ast.body);

    quote!{
        impl #ident #generics #where_clause {
            #[allow(unused_variables)]
            pub fn vertex_attrib_pointers(gl: &::gl::Gl) {
                let stride = ::std::mem::size_of::<Self>();
                let offset = 0;

                #(#fields_vertex_attrib_pointer)*
            }
        }
    }
}

fn generate_vertex_attrib_pointer_calls(body: &syn::Body) -> Vec<quote::Tokens> {
    match body {
        &syn::Body::Enum(_) => panic!("VertexAttribPointers can not be implemented for enums"),
        &syn::Body::Struct(syn::VariantData::Unit) => {
            panic!("VertexAttribPointers can not be implemented for Unit structs")
        }
        &syn::Body::Struct(syn::VariantData::Tuple(_)) => {
            panic!("VertexAttribPointers can not be implemented for Tuple structs")
        }
        &syn::Body::Struct(syn::VariantData::Struct(ref s)) => s
            .iter()
            .map(generate_struct_field_vertex_attrib_pointer_call)
            .collect(),
    }
}

fn generate_struct_field_vertex_attrib_pointer_call(field: &syn::Field) -> quote::Tokens {
    let field_name = match field.ident {
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };
    let location_attr = field
        .attrs
        .iter()
        .filter(|a| a.value.name() == "location")
        .next()
        .unwrap_or_else(|| panic!("Field {} is missing #[location = ?] attribute", field_name));

    let location_value: usize = match location_attr.value {
        syn::MetaItem::NameValue(_, syn::Lit::Str(ref s, _)) => s.parse().unwrap_or_else(|_| {
            panic!(
                "Field {} location attribute value must contain an integer",
                field_name
            )
        }),
        _ => panic!(
            "Field {} location attribute value must be a string literal",
            field_name
        ),
    };

    let field_ty = &field.ty;
    quote! {
        let location = #location_value;
        unsafe {
            #field_ty::vertex_attrib_pointer(gl, stride, location, offset);
        }
        let offset = offset + ::std::mem::size_of::<#field_ty>();
    }
}
