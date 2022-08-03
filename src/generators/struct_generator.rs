use proc_macro2::*;

use super::EcsSoa;

use quote::{format_ident, quote};

pub fn generate_structs(
    entity_signatures: &Vec<(Ident, Vec<(bool, Ident)>)>,
    component_labels: &Vec<(Ident, Ident)>,
    soa_structs: &mut Vec<EcsSoa>,
) -> TokenStream {
    let mut struct_output = TokenStream::new();
    let mut ecs_output = TokenStream::new();

    let mut entity_soa_idents = Vec::new();
    let mut entity_idents = Vec::new();
    let mut ecs_fields = Vec::new();

    let mut deleters = Vec::new();

    let mut labels = Vec::new();
    let mut types = Vec::new();
    for c in component_labels {
        labels.push(c.0.clone());
        types.push(c.1.clone());
    }

    for entity in entity_signatures.iter() {
        let entity_type = entity.0.clone();
        let ecs_field = quote::format_ident!("{}_soa", entity.0.to_string().to_lowercase());
        entity_soa_idents.push(entity_type.clone());
        entity_idents.push(entity.0.clone());
        ecs_fields.push(ecs_field.clone());

        let mut field_soa_idents = Vec::new();
        let mut field_idents = Vec::new();
        let mut field_types = Vec::new();
        let mut field_ident_locks = Vec::new();

        let mut soa_fields = Vec::new();

        for i in entity.1.iter() {
            let t = component_labels
                .iter()
                .position(|x| i.1.cmp(&x.0).is_eq())
                .unwrap();

            let lc_ident = i.1.to_string().to_lowercase();
            field_idents.push(quote::format_ident!("{}", lc_ident));
            field_ident_locks.push(format_ident!("{}_lock", lc_ident));

            let fd = quote::format_ident!("{}", lc_ident);
            let tp = component_labels[t].1.clone();

            field_soa_idents.push(fd.clone());
            field_types.push(tp.clone());

            soa_fields.push((fd, tp));
        }

        struct_output.extend(quote! {
            #[derive(Default,Debug,Clone)]
            pub struct #entity_type {
                #( #field_soa_idents : Arc<RwLock<Vec< #field_types >>> ,)*

                generation: Arc<RwLock<Vec<u32>>>,
                entity_state: Arc<RwLock<Vec<EntityState>>>,

                free_head: usize
            }

        });

        let a_name = quote::format_ident!("add_{}", ecs_field);
        let d_name = quote::format_ident!("delete_{}", ecs_field);

        ecs_output.extend(quote! {

            fn #d_name (&mut self, key: &Key) -> Option<Entity> {

                #( let mut #field_ident_locks = self.#ecs_field . #field_idents .write().ok()? ; )*
                let mut generation_lock = self.#ecs_field.generation.write().ok()?;
                let mut entity_state_lock = self.#ecs_field.entity_state.write().ok()?;

                let g = generation_lock.get(key.index)?;
                let e = entity_state_lock.get(key.index)?;

                if *g == key.generation && *e == EntityState::Occupied {

                    let mut e = Entity{
                        #( #labels : None ),* ,
                        entity_type: key.entity_type.clone()
                    };

                    #(
                        e. #field_idents = Some(#field_ident_locks [key.index] .clone());
                    )*

                    entity_state_lock[key.index] = EntityState::Free{next_free: self. #ecs_field . free_head};
                    self. #ecs_field .free_head = key.index;

                    Some(e)

                }else {
                    None
                }
            }

            pub fn #a_name (&mut self, #(#field_idents : #field_types ,)*) -> Option<Key> {


                #( let mut #field_ident_locks = self.#ecs_field . #field_idents .write().ok()? ; )*
                let mut generation_lock = self.#ecs_field.generation.write().ok()?;
                let mut entity_state_lock = self.#ecs_field.entity_state.write().ok()?;

                let free_head = self.#ecs_field.free_head;

                if free_head == usize::MAX {

                    #( #field_ident_locks.push( #field_idents ) ; )*
                    generation_lock.push(0);
                    entity_state_lock.push(EntityState::Occupied);

                    Some(Key {
                        index: entity_state_lock.len() -1 ,
                        generation: 0,
                        entity_type: EntityType:: #entity_type
                    })
                }else{
                    #( #field_ident_locks[free_head] = #field_idents ; )*

                    generation_lock[free_head] += 1;

                    let t = if let EntityState::Free{next_free} = entity_state_lock[free_head] {
                        next_free
                    }else{
                        panic!("free_head is fucked");
                    };
                    entity_state_lock[free_head] = EntityState::Occupied;

                    self.#ecs_field.free_head = t;

                    Some(Key {
                        index: free_head ,
                        generation: generation_lock[free_head],
                        entity_type: EntityType:: #entity_type
                    })
                }
            }
        });

        deleters.push(d_name);

        let ecs_soa = EcsSoa {
            name: (ecs_field, entity_type),
            fields: soa_fields,
        };
        soa_structs.push(ecs_soa);
    }

    struct_output = quote! {
        #[derive(Debug,Clone)]
        pub struct CHAINED_ECS{
            #(pub #ecs_fields : #entity_soa_idents ),*
        }

        impl CHAINED_ECS{
            pub fn new() -> Self {
                #(let mut #ecs_fields = #entity_soa_idents ::default() ; )*

                #(#ecs_fields .free_head = usize::MAX ; )*
                CHAINED_ECS{
                    #(#ecs_fields ),*
                }
            }

            #ecs_output

            pub fn delete(&mut self, key: &Key) -> Option<Entity> {

                match key.entity_type{
                    #(EntityType:: #entity_idents => {

                        self. #deleters (key)

                    } ),*
                }

            }
        }

        pub struct Entity{
            #( #labels : Option< #types > ),* ,
            entity_type: EntityType
        }

        #[derive(Debug,Clone,Copy)]
        pub enum EntityType{
            #( #entity_idents ),*
        }

        #[derive(Debug,Clone)]
        pub struct Key {
            index: usize,
            generation: u32,
            entity_type: EntityType
        }

        #[derive(Debug, PartialEq, Eq)]
        pub enum EntityState{
            Free{ next_free: usize },
            Occupied
        }

        #struct_output
    };

    struct_output
}
