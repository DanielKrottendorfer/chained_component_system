use proc_macro2::*;
use quote::quote;

use super::EcsSoa;

pub fn generate_accessors(
    component_labels: &Vec<(Ident, Ident)>,
    system_signatures: &Vec<(Ident, Vec<Ident>)>,
    ecs_soas: &Vec<EcsSoa>,
) -> TokenStream {
    let mut output = TokenStream::new();
    let mut output2 = TokenStream::new();

    for system_sig in system_signatures {
        let accessor_name = quote::format_ident!("{}Accessor", system_sig.0);
        let iterator_name = quote::format_ident!("{}Iterator", system_sig.0);

        let s: String = accessor_name
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

        let mut soa_fits = Vec::new();

        'ecsloop: for e in ecs_soas {
            for f in system_sig.1.iter() {
                if !e.fields.iter().any(|x| x.0 == *f) {
                    continue 'ecsloop;
                }
            }
            soa_fits.push(e.name.0.clone());
        }

        if soa_fits.is_empty() {
            continue;
        }

        let component_types: Vec<Ident> = system_sig
            .1
            .iter()
            .map(|x| {
                let i = component_labels.iter().find(|y| y.0 == *x).unwrap();
                i.1.clone()
            })
            .collect();

        output.extend(build_accessor_struct(
            &accessor_name,
            &iterator_name,
            &system_sig.1,
            &component_types,
            soa_fits.len(),
        ));

        output2.extend(build_accessor_constructors(
            &fn_name,
            &accessor_name,
            &system_sig.1,
            &soa_fits,
        ));
    }

    output.extend(quote! {
        impl CHAINED_ECS{
            #output2
        }
    });

    output
}

fn build_accessor_struct(
    accessor_name: &Ident,
    iterator_name: &Ident,
    field_names: &Vec<Ident>,
    field_types: &Vec<Ident>,
    count: usize,
) -> TokenStream {
    let first_fn = field_names[0].clone();

    let mut arrays = Vec::new();

    for f_n in field_names {
        let mut t = Vec::new();

        for c in 0..count {
            t.push(quote! {
                self.#f_n [#c] .lock().unwrap()
            })
        }

        arrays.push(quote! {
            [ #(#t),*]
        });
    }

    let t = quote! {

        #[derive(Debug)]
        pub struct #accessor_name {
            #(  #field_names : [ Arc<Mutex<Vec< #field_types >>> ;  #count ] ),*
        }

        impl #accessor_name {
            pub fn iter(&mut self) -> #iterator_name {
                #iterator_name {
                    #( #field_names: #arrays ),* ,
                    i: 0,
                    y: 0
                }
            }
        }

        #[derive(Debug)]
        pub struct #iterator_name<'a> {
            #(  #field_names : [ MutexGuard<'a, Vec< #field_types >> ;  #count ] ),* ,
            i: usize,
            y: usize,
        }

        impl<'a> Iterator for #iterator_name <'a> {
            type Item = ( #( &'a mut #field_types  ),* );

            fn next<'b>(&mut self) -> Option<Self::Item> {

                let temp = if self. #first_fn [self.i] .len() > self.y {
                    #( let #field_names = self. #field_names [self.i] .as_mut_ptr() ;)*
                    unsafe {Some(
                        ( #(&mut *  #field_names.add(self.y)),* )
                    )}
                }else {
                    self.y = 0;
                    self.i += 1;
                    if self. #first_fn.len() > self.i {
                        #( let #field_names = self. #field_names [self.i] .as_mut_ptr() ;)*
                        unsafe {Some(
                            ( #(&mut *  #field_names.add(self.y)),* )
                        )}
                    }else{
                        None
                    }
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
    field_names: &Vec<Ident>,
    soa_names: &Vec<Ident>,
) -> TokenStream {
    let mut arrays: Vec<TokenStream> = Vec::new();

    for field in field_names {
        let t = quote!(
            #field: [ #( self. #soa_names . #field .clone() ),* ],
        );
        arrays.push(t);
    }

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
