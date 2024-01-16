extern crate proc_macro;
use core::panic;
use std::sync::atomic::AtomicUsize;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Type, Ident, PatType, ItemStruct, ItemEnum, spanned::Spanned};

static COUNTER : AtomicUsize = AtomicUsize::new(0);

#[proc_macro_attribute]
pub fn margarine(_: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse2(item.clone().into()) {
        Ok(it) => return margarine_function(it),
        Err(_) => (),
    };


    match syn::parse2(item.clone().into()) {
        Ok(it) => return margarine_struct(it),
        Err(_) => (),
    };


    match syn::parse2(item.clone().into()) {
        Ok(it) => return margarine_enum(it),
        Err(_) => (),
    };

    panic!("invalid item");
}


fn margarine_function(func: ItemFn) -> TokenStream {
    let name = func.sig.ident;
    let inputs = func.sig.inputs;
    let ret = func.sig.output;
    let ret = match ret {
        syn::ReturnType::Default => Type::Tuple(syn::parse2(quote!(())).unwrap()),
        syn::ReturnType::Type(_, v) => *v,
    };
    let body = func.block;

    let struct_name = format!("__Args_{}", name.to_string());
    let struct_name = Ident::new(&struct_name, name.span());
    let mut vec = vec![];
    for v in &inputs {
        let ident = match v {
            syn::FnArg::Receiver(_) => todo!(),
            syn::FnArg::Typed(PatType { pat, .. }) => match *pat.clone() {
                syn::Pat::Ident(v) => v,
                _ => todo!(),
            },
        };

        vec.push(ident.ident);
    }

    let quote = quote!(
        #[repr(C)]
        #[derive(Clone, Copy, Debug, PartialEq)]
        struct #struct_name {
            #inputs,
            __ret: #ret,
        }

        #[no_mangle]
        pub extern "C" fn #name(ctx: &Ctx, __argp: *mut #struct_name) {
            let (#(#vec),*) = {
                let data = unsafe { &*__argp };
                (#(data.#vec),*)
            };

            let ret = || -> #ret { #body };
            let ret = ret();

            unsafe { *__argp }.__ret = ret;
        }
    );

    quote.into()
}


fn margarine_struct(st: ItemStruct) -> TokenStream {
    quote! {
        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        #st
    }.into()
}


fn margarine_enum(et: ItemEnum) -> TokenStream {
    let name = et.ident;
    let data_union = format!("__EnumData_{}", name);
    let data_union = Ident::new(&data_union, name.span());

    let mut outside = quote!();
    let mut union_fields = quote!();
    let mut impls = quote!();
    for (id, v) in et.variants.iter().enumerate() {
        let id = id as u32;
        let name = &v.ident;
        let as_name = format!("as_{}", name.to_string());
        let as_name = Ident::new(&as_name, name.span());
        let ty = &v.fields;
        match ty {
            syn::Fields::Named(v) => {
                let time = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Acquire);
                let time = format!("_{time}");
                let time = Ident::new(&time, name.span());

                let mut names = Vec::with_capacity(v.named.len());
                for v in &v.named {
                    names.push(&v.ident);
                }

                let mut tys = Vec::with_capacity(v.named.len());
                for v in &v.named {
                    tys.push(&v.ty);
                }

                outside = quote!(
                    #outside
                    
                    #[margarine]
                    struct #time #v
                );


                union_fields = quote!(
                    #union_fields
                    #name: #time,
                );


                impls = quote!(
                    #impls

                    pub fn #name(#(#names: #tys),*) -> Self {
                        Self {
                            __tag: #id,
                            __data: #data_union {
                                #name: #time { 
                                    #(#names),*
                                },
                            }
                        }
                    }


                    pub fn #as_name(self) -> Option<(#(#tys),*)> {
                        if self.__tag != #id { return None }
                        let data = unsafe { self.__data.#name };
                        Some((#(data.#names),*))
                    }
                );
            },


            syn::Fields::Unnamed(v) => {
                let time = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Acquire);
                let time = format!("_{time}");
                let time = Ident::new(&time, name.span());

                let mut names = Vec::with_capacity(v.unnamed.len());
                for v in v.unnamed.iter().enumerate() {
                    names.push(Ident::new(&format!("_{}", v.0), v.1.span()));
                }

                let mut tys = Vec::with_capacity(v.unnamed.len());
                for v in &v.unnamed {
                    tys.push(&v.ty);
                }

                outside = quote!(
                    #outside

                    #[margarine]
                    struct #time {
                        #(#names: #tys),*
                    }
                );


                union_fields = quote!(
                    #union_fields
                    #name: #time,
                );


                impls = quote!(
                    #impls

                    pub fn #name(#(#names: #tys),*) -> Self {
                        Self {
                            __tag: #id,
                            __data: #data_union {
                                #name: #time { 
                                    #(#names),*
                                },
                            }
                        }
                    }


                    pub fn #as_name(self) -> Option<(#(#tys),*)> {
                        if self.__tag != #id { return None }
                        let data = unsafe { self.__data.#name };
                        Some((#(data.#names),*))
                    }
                );
 
            },
            

            syn::Fields::Unit => {
                union_fields = quote!(
                    #union_fields
                    #name: u8,
                );


                impls = quote!(
                    #impls
                    
                    #[allow(non_upper_case_globals)]
                    pub const #name : Self = Self {
                        __tag: #id,
                        __data: #data_union { #name: 0 },
                    };


                    pub fn #as_name(self) -> Option<()> {
                        if self.__tag != #id { return None }
                        Some(())
                    }
                );
            },
        }
    }
    
    let mut names = Vec::with_capacity(et.variants.len());
    let iter = 0..et.variants.len() as u32;
    let iter2 = 0..et.variants.len() as u32;
    for v in et.variants { names.push(v.ident) }

    
    quote!{
        #[derive(Clone, Copy)]
        #[repr(C)]
        #[allow(non_snake_case)]
        union #data_union {
            #union_fields
        }
        

        #[repr(C)]
        #[derive(Clone, Copy)]
        struct #name {
            __tag: u32,
            __data: #data_union,
        }


        #[allow(non_snake_case)]
        impl #name {
            #impls
        }
        
        
        impl ::std::fmt::Debug for #name {
           fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                let mut dbg = f.debug_struct(stringify!(#name));

                match self.__tag {
                    #(
                        #iter => dbg.field(stringify!(#names), unsafe { &self.__data.#names }),
                    )*
                    _ => panic!("unknown variant"),
                };
                
                dbg.finish()
           }
        }


        impl std::cmp::PartialEq for #name {
            fn eq(&self, oth: &Self) -> bool {
                if self.__tag != oth.__tag { return false }

                match self.__tag {
                    #(
                        #iter2 => unsafe { self.__data.#names == oth.__data.#names },
                    )*
                    _ => panic!("unknown variant"),
                }
            }
        }



        #outside
    }.into()
}


