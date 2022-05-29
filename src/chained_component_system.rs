use std::borrow::{BorrowMut};

use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::quote;

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

#[derive(Debug)]
pub struct EcsSoa {
    pub ecs_field: (Ident, Ident),
    pub soa_fields: Vec<(Ident, Ident)>,
}

pub fn generate_structs(
    entity_signatures: &Vec<(Ident, Vec<Ident>)>,
    component_labels: &Vec<(Ident, Ident)>,
    soa_structs: &mut Vec<EcsSoa>,
) -> TokenStream {
    let mut output = TokenStream::new();

    let mut entity_idents = Vec::new();
    let mut ecs_fields = Vec::new();

    for entity in entity_signatures.iter() {
        let entity_ident = quote::format_ident!("{}SOA", entity.0.clone());
        let ecs_field = quote::format_ident!("{}_soa", entity.0.to_string().to_lowercase());
        entity_idents.push(entity_ident.clone());
        ecs_fields.push(ecs_field.clone());

        let mut field_soa_idents = Vec::new();
        let mut field_idents = Vec::new();
        let mut field_types = Vec::new();

        let mut soa_fields = Vec::new();

        for i in entity.1.iter() {
            let t = component_labels
                .iter()
                .position(|x| i.cmp(&x.0).is_eq())
                .unwrap();

            let lc_ident = i.to_string().to_lowercase();
            field_idents.push(quote::format_ident!("{}", lc_ident));

            let fd = quote::format_ident!("{}", lc_ident);
            let tp = component_labels[t].1.clone();

            field_soa_idents.push(fd.clone());
            field_types.push(tp.clone());

            soa_fields.push((fd, tp));
        }

        output.extend(quote! {
            #[derive(Default,Debug)]
            pub struct #entity_ident {
                pub #( #field_soa_idents : Vec< #field_types > ,)*
            }

        });

        let f_name = quote::format_ident!("new_{}", ecs_field);
        output.extend(quote! {
            impl #entity_ident{
                pub fn #f_name (&mut self, #(#field_idents : #field_types ,)* ) {
                    #(self. #field_soa_idents .push( #field_idents) ;)*
                }
            }
        });

        let ecs_soa = EcsSoa {
            ecs_field: (ecs_field, entity_ident),
            soa_fields,
        };
        soa_structs.push(ecs_soa);
    }

    output.extend(quote! {
        #[derive(Default,Debug)]
        pub struct CHAINED_ECS{
            #(pub #ecs_fields : #entity_idents ,)*
        }
    });

    output
}

pub fn generate_chunk_iters(
    component_labels: &Vec<(Ident, Ident)>,
    system_signatures: &Vec<(Ident, Vec<Ident>)>,
    ecs_soas: &Vec<EcsSoa>,
) -> TokenStream {
    let mut output = TokenStream::new();

    for system_sig in system_signatures {
        let chunk_iter_name = quote::format_ident!("{}ChunkIterator", system_sig.0);

        let s: String = chunk_iter_name
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

        let fn_name = quote::format_ident!("get{}", s);

        let mut component_tp: Vec<Ident> = Vec::new();

        for x in system_sig.1.iter() {
            component_tp.push(
                component_labels
                    .iter()
                    .find(|y| y.0.cmp(x).is_eq())
                    .unwrap()
                    .1
                    .clone(),
            );
        }

        output.extend(build_chunk_iter_struct(
            &chunk_iter_name,
            &system_sig.1,
            &component_tp,
        ));

        let mut chain = TokenStream::new();
        let mut return_iter = TokenStream::new();

        let mut did_chain = false;

        for ecs_soa in ecs_soas.iter() {
            if system_sig.1.iter().all(|x| {
                ecs_soa
                    .soa_fields
                    .iter()
                    .find(|y| y.0.cmp(x).is_eq())
                    .is_some()
            }) {
                output.extend(build_into_iter(
                    &fn_name,
                    &chunk_iter_name,
                    &ecs_soa.ecs_field.1,
                    &system_sig.1,
                    &component_tp,
                ));

                let ecs_field = ecs_soa.ecs_field.0.clone();

                if chain.is_empty() {
                    chain = quote! {
                        self. #ecs_field . #fn_name ()
                    };
                    return_iter = quote! {
                        #chunk_iter_name
                    };
                } else {
                    did_chain = true;
                    chain = quote! {
                        #chain .chain(self. #ecs_field . #fn_name ())
                    };
                    return_iter = quote! {
                        Chain< #return_iter, #chunk_iter_name >
                    };
                }
            }
        }

        if !chain.is_empty() {
            println!("+++++++++++\n{}\n++++++++++++++++", return_iter.to_string());
            if did_chain {
                output.extend(quote! {
                    impl<'a> CHAINED_ECS {
                        pub fn #fn_name(&mut self) -> #return_iter {
                            #chain
                        }
                    }
                });
            } else {
                output.extend(quote! {
                    impl CHAINED_ECS {
                        pub fn #fn_name (&mut self) ->  #return_iter{
                            #chain
                        }
                    }
                });
            }
        }
    }

    output
}

fn build_into_iter(
    fn_name: &Ident,
    chunk_iter_name: &Ident,
    soa_name: &Ident,
    field_names: &Vec<Ident>,
    field_types: &Vec<Ident>,
) -> TokenStream {
    if field_names.len() != field_types.len() {
        panic!("you done goofed");
    }

    quote! {
        impl #soa_name {

            pub fn #fn_name (&mut self) -> #chunk_iter_name {
                #chunk_iter_name {
                    index: 0,
                    #(  #field_names : &mut self. #field_names ,)*
                }
            }
        }
    }
}

fn build_chunk_iter_struct(
    chunk_iter_name: &Ident,
    field_names: &Vec<Ident>,
    field_types: &Vec<Ident>,
) -> TokenStream {
    if field_names.len() != field_types.len() {
        panic!("you done goofed");
    }

    let first_fn = field_names[0].clone();

    quote! {
        pub struct #chunk_iter_name <'a> {
            #(#field_names: & 'a mut Vec< #field_types > ,)*

            index: usize
        }

        impl<'a> Iterator for #chunk_iter_name <'a> {
            type Item = ( #( &'a mut #field_types , )* );

            fn next<'b>(&mut self) -> Option<Self::Item> {

                let t = if self. #first_fn .len() > self.index {
                    #( let #field_names = self. #field_names .as_mut_ptr() ;)*
                    unsafe {Some((
                        #( &mut * #field_names .add(self.index) ,)*)
                    )}

                } else {
                    None
                };
                self.index += 1;
                t
            }
        }
    }
}
