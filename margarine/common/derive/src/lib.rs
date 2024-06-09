use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(ImmutableData)]
pub fn immutable(ts: TokenStream) -> TokenStream {
    let strct : syn::ItemStruct = syn::parse(ts).unwrap();

    let name = strct.ident;
    let gens = strct.generics;
    let field_names = strct.fields.iter().map(|x| x.ident.as_ref().unwrap());
    let field_tys = strct.fields.iter().map(|x| &x.ty);
    quote! {
        impl #gens #name #gens {
            #(
                #[inline(always)]
                pub fn #field_names(self) -> #field_tys {
                    self.#field_names
                }
            )*
        }
    }.into()
}
