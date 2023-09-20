use proc_macro2::*;
use quote::quote;

use super::{to_snake_ident, EcsSoa};

pub fn generate_accessors(
    component_labels: &Vec<(Ident, TokenStream)>,
    system_signatures: &Vec<(Ident, Vec<(bool, Ident)>)>,
    ecs_soas: &Vec<EcsSoa>,
) -> TokenStream {
    let mut output = TokenStream::new();
    let mut output2 = TokenStream::new();

    for system_sig in system_signatures {
        let mut soa_fits = Vec::new();
        let mut entity_fits = Vec::new();


        let mut is_mutable = false;

        'ecsloop: for e in ecs_soas {
            for f in system_sig.1.iter() {
                if f.0 {
                    is_mutable = true;
                }

                if f.1 == key {
                    continue;
                }

                if !e.fields.iter().any(|x| x.0 == f.1) {
                    continue 'ecsloop;
                }
            }
            soa_fits.push(e.name.0.clone());
            entity_fits.push(e.name.1.clone());
        }

        if soa_fits.is_empty() {
            continue;
        }

        let accessor_name = quote::format_ident!("{}Accessor", system_sig.0);
        let lock_name = quote::format_ident!("{}Lock", system_sig.0);
        let iterator_name = quote::format_ident!("{}Iterator", system_sig.0);

        let fn_name = quote::format_ident!("get{}", to_snake_ident(&accessor_name));

        let mut system_s = Vec::new();
        let component_types: Vec<TokenStream> = system_sig
            .1
            .iter()
            .filter_map(|x| {
                system_s.push(x.clone());
                let i = component_labels.iter().find(|y| y.0 == x.1).unwrap();
                Some(i.1.clone())
            })
            .collect();

        output.extend(build_accessor_struct(
            &accessor_name,
            &iterator_name,
            &lock_name,
            &system_s,
            &component_types,
            &entity_fits,
            keyed,
            is_mutable,
            soa_fits.len(),
        ));

        output2.extend(build_accessor_constructors(
            &fn_name,
            &accessor_name,
            &system_s,
            &soa_fits,
        ));
    }

    output.extend(quote! {
        #[allow(dead_code,non_camel_case_types)]
        impl CHAINED_ECS{
            #output2
        }
    });

    output
}

fn build_accessor_struct(
    accessor_name: &Ident,
    iterator_name: &Ident,
    lock_name: &Ident,
    field_names: &Vec<(bool, Ident)>,
    field_types: &Vec<TokenStream>,
    entity_names: &Vec<Ident>,
    keyed: bool,
    is_mutable: bool,
    count: usize,
) -> TokenStream {
    let mut arrays = Vec::new();
    let indices: Vec<usize> = (0..entity_names.len()).map(|x| x).collect();

    for f_n in field_names {
        let mut t = Vec::new();
        let f_name = &f_n.1;
        if f_n.0 {
            for c in 0..count {
                t.push(quote! {
                    self.#f_name [#c] .write().unwrap()
                })
            }
        } else {
            for c in 0..count {
                t.push(quote! {
                    self.#f_name [#c] .read().unwrap()
                })
            }
        }
        arrays.push(quote! {
            [ #(#t),*]
        });
    }

    let mut iter_types = Vec::new();
    let mut iter_ptr_types = Vec::new();
    let mut iter_mut_tpes = Vec::new();
    let lock_types: Vec<TokenStream> = field_names
        .iter()
        .zip(field_types.iter())
        .map(|(f_name, f_type)| {
            let _field_name = &f_name.1;
            if f_name.0 {
                iter_types.push(quote! {
                    mut #f_type
                });
                iter_ptr_types.push(quote! {
                    as_mut_ptr()
                });
                iter_mut_tpes.push(quote! {
                    &mut *
                });
                quote! {
                    RwLockWriteGuard<'a, Vec< #f_type >>
                }
            } else {
                iter_types.push(quote! {
                    #f_type
                });
                iter_ptr_types.push(quote! {
                    as_ptr()
                });
                iter_mut_tpes.push(quote! {
                    & *
                });

                quote! {
                    RwLockReadGuard<'a, Vec< #f_type >>
                }
            }
        })
        .collect();

    let mut t = Vec::new();
    for c in 0..count {
        t.push(quote! {
            self.generations[#c] .read().unwrap()
        })
    }
    let generations = quote! {
        [ #(#t),*]
    };

    let mut t = Vec::new();
    for c in 0..count {
        t.push(quote! {
            self.entity_states[#c] .read().unwrap()
        })
    }
    let entity_states = quote! {
        [ #(#t),*]
    };


    let mut maybe_mutable = TokenStream::new();
    if is_mutable {
        maybe_mutable = quote! {
            mut
        }
    }

    let field_names: Vec<Ident> = field_names.iter().map(|x| x.1.clone()).collect();

    let t = quote! {

        #[derive(Debug)]
        pub struct #accessor_name {
            #( #field_names :   [ Arc<RwLock<Vec< #field_types >>> ;  #count ] , )*

            generations:        [ Arc<RwLock<Vec<u32>>>;  #count ],
            entity_states:      [ Arc<RwLock<Vec<EntityState>>>;  #count ],
        }

        impl #accessor_name {
            pub fn lock(& #maybe_mutable self) -> #lock_name {
                #lock_name {
                    #( #field_names: #arrays ,)*

                    entity_types: [ #( EntityType:: #entity_names ),* ] ,

                    generations: #generations,
                    entity_states: #entity_states,

                }
            }
        }

        #[derive(Debug)]
        pub struct #lock_name<'a> {
            #( #field_names :   [ #lock_types ;  #count ], )*

            entity_types:       [ EntityType; #count ],
            generations:        [ RwLockReadGuard<'a, Vec<u32>>;  #count ],
            entity_states:      [ RwLockReadGuard<'a, Vec<EntityState>>;  #count ],
        }

        impl<'a> #lock_name<'a> {
            pub fn iter<'b>(&'b #maybe_mutable self) -> #iterator_name <'a,'b>{
                #iterator_name {
                    #( #field_names:    &#maybe_mutable self. #field_names ),* ,

                    entity_types:  & self.entity_types,
                    generations:   & self.generations,
                    entity_states: & self.entity_states,

                    i: 0,
                    y: 0
                }
            }
            pub fn get<'b>(&'b #maybe_mutable self,key: Key) -> Option<( #( &'a #iter_types  ),* )> {

                let i = match key.entity_type {
                    #(
                        EntityType:: #entity_names => {

                            #indices
                        }
                    ),*
                    _ => return None
                };

                if self.entity_states[i][key.index] == EntityState::Occupied {
                    if self.generations[i][key.index] == key.generation{
                        #( let #field_names = self. #field_names [i] . #iter_ptr_types ;)*
                        unsafe {Some(
                            ( #(#iter_mut_tpes  #field_names.add(key.index) ),*)
                        )}
                    }else {
                        None
                    }
                }else{
                    None
                }
            }
        }

        #[derive(Debug)]
        pub struct #iterator_name<'a,'b> {
            #(  #field_names :  &'b #maybe_mutable [ #lock_types ;  #count ] ,)*

            entity_types:       &'b [ EntityType; #count ],
            generations:        &'b [ RwLockReadGuard<'a, Vec<u32>>;  #count ],
            entity_states:      &'b [ RwLockReadGuard<'a, Vec<EntityState>>;  #count ],

            i: usize,
            y: usize
        }

        impl<'a,'b> Iterator for #iterator_name <'a,'b> {
            type Item = ( #( &'a #iter_types  ),*);

            fn next(&mut self) -> Option<Self::Item> {

                loop {
                    while self. entity_states [self.i] .len() <= self.y {
                        self.y = 0;
                        self.i += 1;
                        if self.i == self. entity_states .len(){
                            return None
                        }
                    }

                    match self.entity_states[self.i][self.y] {
                        EntityState::Occupied => break,
                        EntityState::Free{..} => self.y +=1
                    };
                }

                let temp = {

                    #( let #field_names = self. #field_names [self.i] . #iter_ptr_types ;)*
                    unsafe {Some(
                        ( #(#iter_mut_tpes  #field_names.add(self.y) ),*)
                    )}
                };

                self.y += 1;
                temp
            }
        }

    };

    t
}

fn build_accessor_constructors(
    fn_name: &Ident,
    accessor_name: &Ident,
    field_names: &Vec<(bool, Ident)>,
    soa_names: &Vec<Ident>,
) -> TokenStream {
    let mut arrays: Vec<TokenStream> = Vec::new();

    let field_names: Vec<Ident> = field_names.iter().map(|x| x.1.clone()).collect();

    for field in field_names {
        let t = quote!(
            #field: [ #( self. #soa_names . #field .clone() ),* ] ,
        );
        arrays.push(t);
    }

    arrays.push(quote! {
        generations: [ #(self. #soa_names .generation.clone()),* ],
        entity_states: [ #(self. #soa_names .entity_state.clone()),* ]
    });

    let t = quote! {
        pub fn #fn_name (&self) -> #accessor_name{
            #accessor_name {
                #(
                    #arrays
                )*
            }
        }
    };

    t
}
