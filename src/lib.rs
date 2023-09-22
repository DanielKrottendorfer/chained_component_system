use std::borrow::BorrowMut;

use generators::{
    accessor_generator::generate_accessors, entity_accessor_generator::generate_entity_accessor,
    struct_generator::generate_structs,
};
use parsers::*;
use proc_macro2::TokenTree;
use quote::quote;

mod generators;
mod parsers;

#[proc_macro]
pub fn chained_component_system(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = proc_macro2::TokenStream::from(_item).into_iter();

    let mut component_labels = None;
    let mut entity_signatures = None;
    let mut system_signatures = None;

    loop {
        let section: Vec<TokenTree> = input
            .borrow_mut()
            .take_while(|x| match x {
                proc_macro2::TokenTree::Punct(p) => p.as_char() != ';',
                _ => true,
            })
            .collect();
        if section.is_empty() {
            break;
        }

        let section_ident = &section[0];
        let group = match &section[1] {
            TokenTree::Group(g) => g,
            _ => panic!("Brackets are missing"),
        };

        match section_ident {
            TokenTree::Ident(i) => match i.to_string().as_str() {
                "components" => {
                    component_labels = Some(parse_component_labels(group));
                }
                "entities" => {
                    entity_signatures = Some(parse_signatures(group));
                }
                "global_systems" => {
                    system_signatures = Some(parse_signatures(group));
                }
                _ => panic!("unrecogniced section name"),
            },
            _ => panic!("section label missing"),
        }
    }

    let component_labels = component_labels.expect("components missing");
    let entity_signatures = entity_signatures.expect("entities missing");
    let system_signatures = system_signatures.expect("global_systems missing");
    let mut ecs_soas = Vec::new();
    let soas = generate_structs(&entity_signatures, &component_labels, &mut ecs_soas);
    let accessors = generate_accessors(&component_labels, &system_signatures, &ecs_soas);
    let e_accessors = generate_entity_accessor(&ecs_soas);
    let output = quote! {
        #soas
        #accessors
        #e_accessors
    };

    #[cfg(target_feature = "output_foo")]
    {
        use std::{fs::File, io::Write};
        let mut file = File::create("foo.txt").unwrap();
        file.write_all(output.to_string().as_bytes()).unwrap();
    }
    output.into()
}
