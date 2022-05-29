use std::borrow::BorrowMut;

use chained_component_system::{generate_structs, parse_signatures};
use proc_macro2::{TokenTree};
use quote::quote;

use crate::chained_component_system::{generate_chunk_iters, parse_component_labels};

mod chained_component_system;

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
    let chunks = generate_chunk_iters(&component_labels, &system_signatures, &ecs_soas);

    let output = quote! {
        #soas
        #chunks
    };

    println!("{}", output.to_string());
    output.into()
}
