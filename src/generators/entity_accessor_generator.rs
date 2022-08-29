use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use super::{to_snake_ident, EcsSoa};

pub fn generate_entity_accessor(ecs_soas: &Vec<EcsSoa>) -> TokenStream {
    let mut out = TokenStream::new();
    let mut out2 = TokenStream::new();

    for ecs_soa in ecs_soas {
        let ecs_field_name = &ecs_soa.name.0;
        let entity_name = &ecs_soa.name.1;

        let iterator_name = format_ident!("{}Iterator", ecs_soa.name.1);
        let lock_name = format_ident!("{}Lock", ecs_soa.name.1);
        let fn_name = quote::format_ident!("get{}", to_snake_ident(entity_name));

        let field_names: Vec<Ident> = ecs_soa.fields.iter().map(|x| x.0.clone()).collect();
        let types: Vec<TokenStream> = ecs_soa.fields.iter().map(|x| x.1.clone()).collect();

        out.extend(quote! {

            impl #entity_name{
                pub fn lock(&mut self) -> #lock_name{
                    #lock_name{
                        #( #field_names : self. #field_names .write().unwrap() ,)*
                        generation: self.generation.read().unwrap(),
                        entity_state: self.entity_state.read().unwrap(),
                    }
                }
            }

            pub struct #lock_name<'a> {
                #( #field_names :   RwLockWriteGuard<'a, Vec< #types >> ,)*

                generation:         RwLockReadGuard<'a,Vec<u32>>,
                entity_state:       RwLockReadGuard<'a,Vec<EntityState>>
            }

            impl<'a> #lock_name<'a> {
                pub fn iter<'b>(&'b mut self) -> #iterator_name <'a,'b>{
                    #iterator_name {
                        #( #field_names:    &mut self. #field_names ,)*
                        entity_state:       &mut self.entity_state,
                        i: 0
                    }
                }

                pub fn get<'b>(&'b mut self,key: &Key) -> Option<( #( &'a #types  ),* )> {

                    if *self.generation.get(key.index)? == key.generation{
                        #( let #field_names = self. #field_names  . as_mut_ptr() ; )*
                        unsafe {Some(
                            ( #( &mut * #field_names.add(key.index) ),*)
                        )}
                    }else {
                        None
                    }

                }
            }


            pub struct #iterator_name<'a,'b> {
                #( #field_names :   &'b mut RwLockWriteGuard<'a, Vec< #types >> ,)*
                entity_state:       &'b mut RwLockReadGuard<'a,Vec<EntityState>>,
                i: usize,
            }

            impl<'a,'b> Iterator for #iterator_name <'a,'b> {
                type Item = ( #( &'a mut #types  ),* );

                fn next(&mut self) -> Option<Self::Item> {

                    loop {
                        if self.i == self.entity_state.len() {
                            return None;
                        }
                        match self.entity_state[self.i] {
                            EntityState::Occupied => break,
                            EntityState::Free { .. } => self.i += 1
                        }
                    }
                    let temp = {

                        #( let #field_names = self. #field_names .as_mut_ptr() ;)*
                        unsafe {Some(
                            ( #(&mut *  #field_names.add(self.i)),*)
                        )}
                    };
                    self.i += 1;
                    temp
                }
            }

        });
        out2.extend(quote! {
            pub fn #fn_name(&mut self) -> #entity_name{
                self.#ecs_field_name.clone()
            }
        })
    }
    out.extend(quote! {
        impl CHAINED_ECS{
            #out2
        }
    });
    out
}
