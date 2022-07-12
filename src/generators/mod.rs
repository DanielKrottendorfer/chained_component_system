use proc_macro2::Ident;

pub mod accessor_generator;
pub mod struct_generator;

#[derive(Debug)]
pub struct EcsSoa {
    pub name: (Ident, Ident),
    pub fields: Vec<(Ident, Ident)>,
}
