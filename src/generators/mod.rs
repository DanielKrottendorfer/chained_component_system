use proc_macro2::{Ident, TokenStream};

pub mod accessor_generator;
pub mod entity_accessor_generator;
pub mod struct_generator;

#[derive(Debug)]
pub struct EcsSoa {
    pub name: (Ident, Ident),
    pub fields: Vec<(Ident, TokenStream)>,
}

pub fn to_snake_ident(i: &Ident) -> Ident {
    let s: String = i
        .to_string()
        .chars()
        .map(|c| {
            if c.is_uppercase() {
                format!("_{}", c.to_lowercase())
            } else {
                c.to_string()
            }
        })
        .collect();

    quote::format_ident!("{}", s)
}
