use proc_macro2::*;

use super::EcsSoa;

use quote::quote;

pub fn generate_structs(
    entity_signatures: &Vec<(Ident, Vec<Ident>)>,
    component_labels: &Vec<(Ident, Ident)>,
    soa_structs: &mut Vec<EcsSoa>,
) -> TokenStream {
    let mut struct_output = TokenStream::new();

    let mut entity_soa_idents = Vec::new();
    let mut entity_idents = Vec::new();
    let mut ecs_fields = Vec::new();

    for entity in entity_signatures.iter() {
        let entity_soa_ident = quote::format_ident!("{}SOA", entity.0.clone());
        let ecs_field = quote::format_ident!("{}_soa", entity.0.to_string().to_lowercase());
        entity_soa_idents.push(entity_soa_ident.clone());
        entity_idents.push(entity.0.clone());
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

        struct_output.extend(quote! {
            #[derive(Default,Debug,Clone)]
            pub struct #entity_soa_ident {
                #(pub  #field_soa_idents : Arc<Mutex<Vec< #field_types >>> ,)*
            }
        });

        let f_name = quote::format_ident!("new_{}", ecs_field);

        struct_output.extend(quote! {
            impl #entity_soa_ident{
                pub fn #f_name (&mut self, #(#field_idents : #field_types ,)* )  -> bool  {
                    #(
                        match self . #field_soa_idents .lock() {
                            Ok(mut l) => {
                                l.push( #field_idents );
                            },
                            Err(a) => {
                                return false;
                            },
                        }
                    )*
                    return true;
                }
            }
        });

        let ecs_soa = EcsSoa {
            name: (ecs_field, entity_soa_ident),
            fields: soa_fields,
        };
        soa_structs.push(ecs_soa);
    }

    struct_output = quote! {
        #[derive(Default,Debug,Clone)]
        pub struct CHAINED_ECS{
            #(pub #ecs_fields : #entity_soa_idents ,)*
        }

        pub enum EntityType{
            #( #entity_idents ,)*
        }

        pub struct Key {
            index: usize,
            generation: u32,
            entity_type: EntityType
        }

        pub enum EntityState{
            Free{ next_free: usize },
            Occupied{ generation: u32 }
        }

        #struct_output
    };

    struct_output
}
