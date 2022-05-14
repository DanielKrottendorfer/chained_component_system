use std::collections::{HashMap, HashSet};

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::quote;

#[proc_macro]
pub fn chained_component_system(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = proc_macro2::TokenStream::from(_item).into_iter();

    let mut componet_map = HashMap::new();
    let mut entity_signatures: Vec<(Ident, HashSet<Ident>)> = Vec::new();
    let mut system_signatures: Vec<(Ident, HashSet<Ident>)> = Vec::new();

    if let TokenTree::Ident(c) = input
        .next()
        .expect("components missing or not defined first")
    {
        assert!(
            c.to_string().starts_with("components"),
            "components missing or not defined first"
        );

        if let TokenTree::Group(g) = input.next().expect("brackets missing") {
            let component_span: Vec<TokenTree> = g.stream().into_iter().map(|x| x).collect();

            let split = component_span.split(|t| {
                if let TokenTree::Punct(p) = t {
                    p.as_char() == ','
                } else {
                    false
                }
            });

            for c in split {
                assert!(c.len() == 3, "missing component definishion");

                let k = if let TokenTree::Ident(i) = &c[0] {
                    i.clone()
                } else {
                    panic!("invlaid component name")
                };

                let v = if let TokenTree::Ident(i) = &c[2] {
                    i.clone()
                } else {
                    panic!("invlaid component type")
                };

                componet_map.insert(k, v);
            }
        }
    }

    if let TokenTree::Ident(c) = input.next().expect("entitys missing or not defined second") {
        assert!(
            c.to_string().starts_with("entitys"),
            "entitys missing or not defined second"
        );

        if let TokenTree::Group(g) = input.next().expect("brackets missing") {
            let component_span: Vec<TokenTree> = g.stream().into_iter().map(|x| x).collect();

            let split = component_span.split(|t| {
                if let TokenTree::Punct(p) = t {
                    p.as_char() == ','
                } else {
                    false
                }
            });

            for tt in split {
                assert!(tt.len() == 2, "missing component definishion");

                let entity_name = if let TokenTree::Ident(i) = &tt[0] {
                    for e in &entity_signatures {
                        if e.0 == *i {
                            panic!("entity name already in use");
                        }
                    }
                    i.clone()
                } else {
                    panic!("invlaid component name")
                };

                let g = if let TokenTree::Group(i) = &tt[1] {
                    i.clone()
                } else {
                    panic!("invlaid component type")
                };

                let mut components = HashSet::new();

                for tt in g.stream() {
                    if let TokenTree::Ident(i) = tt {
                        if componet_map.get(&i).is_some() {
                            if !components.insert(i) {
                                panic!("no duplicate Components allowed")
                            }
                        } else {
                            panic!("component identifiert not defined");
                        }
                    }
                }

                entity_signatures.push((entity_name, components));
            }
        }
    }

    if let TokenTree::Ident(c) = input
        .next()
        .expect("global_systems missing or not defined third")
    {
        assert!(
            c.to_string().starts_with("global_systems"),
            "global_systems missing or not defined third"
        );

        if let TokenTree::Group(g) = input.next().expect("brackets missing") {
            let system_span: Vec<TokenTree> = g.stream().into_iter().map(|x| x).collect();

            let split: Vec<&[TokenTree]> = system_span
                .split(|t| {
                    if let TokenTree::Punct(p) = t {
                        p.as_char() == ','
                    } else {
                        false
                    }
                })
                .collect();

            for tt in split {
                assert!(tt.len() == 2, "wrong system definishion");

                let system_ident = if let TokenTree::Ident(i) = &tt[0] {
                    i.clone()
                } else {
                    panic!()
                };

                let g = if let TokenTree::Group(i) = &tt[1] {
                    i.clone()
                } else {
                    panic!("missing brackets")
                };

                let mut components: HashSet<Ident> = HashSet::new();

                for tt in g.stream() {
                    if let TokenTree::Ident(i) = tt {
                        if componet_map.get(&i).is_some() {
                            if !components.insert(i) {
                                panic!("no duplicate Components allowed")
                            }
                        } else {
                            panic!("component identifiert not defined");
                        }
                    }
                }

                system_signatures.push((system_ident, components));
            }
        }
    }

    let mut soa_structs = TokenStream::new();
    let mut ecs_fields = TokenStream::new();

    for e_s in entity_signatures.iter() {
        let soa_struct_ident = quote::format_ident!("{}SOA", e_s.0);
        let ecs_field_ident = quote::format_ident!("{}_soa", e_s.0.to_string().to_lowercase());

        let field_idents: Vec<Ident> = e_s.1.iter().map(|x| x.clone()).collect();
        let component_idents: Vec<Ident> = e_s
            .1
            .iter()
            .map(|x| componet_map.get(x).unwrap().clone())
            .collect();

        let soa_struct_token = quote!(
            #[derive(Debug,Default)]
            pub struct #soa_struct_ident {
                #(pub #field_idents : Vec<#component_idents> ,)*
            }
        );

        let ecs_field_token = quote!(
            pub #ecs_field_ident : #soa_struct_ident,
        );

        soa_structs.extend(soa_struct_token);
        ecs_fields.extend(ecs_field_token);
    }

    let mut chain_token = TokenStream::new();

    for ss in system_signatures.iter() {
        let mut entity_matches: Vec<Ident> = Vec::new();
        for es in entity_signatures.iter() {
            if ss.1.iter().all(|s| es.1.contains(s)) {
                entity_matches.push(es.0.clone());
            }
        }

        let mut zip_token = TokenStream::new();
        for em in entity_matches.iter() {
            let soa_ident = quote::format_ident!("{}_soa", em.to_string().to_lowercase());

            let mut field_chain = TokenStream::new();
            for field in ss.1.iter() {
                if field_chain.is_empty() {
                    field_chain = quote!(
                        self.#soa_ident.#field.iter()
                    );
                } else {
                    field_chain = quote!(
                        self.#soa_ident.#field.iter().zip(#field_chain)
                    );
                }
            }

            if zip_token.is_empty() {
                zip_token = quote!(
                    #field_chain
                );
            } else {
                zip_token = quote!(
                    #field_chain.chain(#zip_token)
                )
            }
        }
        let mut ret = TokenStream::new();
        for field in ss.1.iter() {
            let s = componet_map.get(field).unwrap();
            ret = quote!(&#s,#ret);
        }

        let ret = quote!((#ret));
        let fn_ident = ss.0.clone();
        chain_token.extend(quote!(
            pub fn #fn_ident(&mut self) -> impl Iterator<Item = #ret>{
                #zip_token
            }
        ));
    }

    let output = quote!(
        #soa_structs

        #[derive(Default,Debug)]
        pub struct ECS{
            #ecs_fields
        }
        impl ECS {
            #chain_token
        }
    );

    println!("{}", output.to_string());

    proc_macro::TokenStream::from(output)
}
