use std::borrow::BorrowMut;

use proc_macro2::*;
use quote::format_ident;

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

pub fn parse_signatures(group: &proc_macro2::Group) -> Vec<(Ident, Vec<(bool, Ident)>)> {
    let mut input = group.stream().into_iter();

    let mut entities = Vec::new();
    let mut_ident = format_ident!("mut");

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

        let mut entity_sigmature = Vec::new();

        match &signature[1] {
            TokenTree::Group(g) => {
                let mut s = g.stream().into_iter();

                loop {
                    let t = s.next();

                    if let Some(f) = t {
                        match f {
                            TokenTree::Ident(i) => {
                                if i.eq(&mut_ident) {
                                    let t = s.next().expect("Component name missing");
                                    if let TokenTree::Ident(i) = t {
                                        entity_sigmature.push((true, i));
                                    } else {
                                        panic!("unexpected symbol at {}", entity_ident.to_string());
                                    }
                                } else {
                                    entity_sigmature.push((false, i));
                                }
                            }
                            TokenTree::Punct(_) => (),
                            _ => panic!("unexpected symbol at {}", entity_ident.to_string()),
                        }
                    } else {
                        break;
                    }
                }
            }
            _ => panic!("Bracket missing"),
        };

        entities.push((entity_ident, entity_sigmature));
    }
    entities
}
