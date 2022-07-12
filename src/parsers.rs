use std::borrow::BorrowMut;

use proc_macro2::*;

pub fn parse_component_labels(g: &Group) -> Vec<(Ident, Ident)> {
    let mut labels = Vec::new();
    let mut input = g.stream().into_iter();

    loop {
        let component_def: Vec<TokenTree> = input
            .borrow_mut()
            .take_while(|x| match x {
                proc_macro2::TokenTree::Punct(p) => p.as_char() != ',',
                _ => true,
            })
            .collect();
        if component_def.is_empty() {
            break;
        }

        let id = match &component_def[0] {
            TokenTree::Ident(i) => i.clone(),
            _ => panic!("missing component ident"),
        };

        let ty = match &component_def[2] {
            TokenTree::Ident(i) => i.clone(),
            _ => panic!("missing component type"),
        };

        labels.push((id, ty));
    }

    labels
}

pub fn parse_signatures(group: &proc_macro2::Group) -> Vec<(Ident, Vec<Ident>)> {
    let mut input = group.stream().into_iter();

    let mut entities = Vec::new();
    loop {
        let signature: Vec<TokenTree> = input
            .borrow_mut()
            .take_while(|x| match x {
                proc_macro2::TokenTree::Punct(p) => p.as_char() != ',',
                _ => true,
            })
            .collect();
        if signature.is_empty() {
            break;
        }

        let entity_ident = match &signature[0] {
            TokenTree::Ident(i) => i.clone(),
            _ => panic!("Entity name missing"),
        };

        let fields = match &signature[1] {
            TokenTree::Group(g) => g.stream().into_iter(),
            _ => panic!("Bracket missing"),
        };

        let mut entity_sigmature = Vec::new();
        for f in fields {
            match f {
                TokenTree::Ident(i) => entity_sigmature.push(i),
                TokenTree::Punct(_) => (),
                _ => panic!("unexpected symbol at {}", entity_ident.to_string()),
            }
        }
        entities.push((entity_ident, entity_sigmature));
    }
    entities
}
