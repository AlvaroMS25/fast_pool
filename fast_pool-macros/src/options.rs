use syn::Ident;
use darling::FromMeta;
use quote::quote;
use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(FromMeta)]
pub struct PoolOptions {
    #[darling(default)]
    pub threads: Option<usize>,
    #[darling(default)]
    pub stack_size: Option<usize>,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub name_fn: Option<Ident>,
    #[darling(default)]
    pub before: Option<Ident>,
    #[darling(default)]
    pub after: Option<Ident>
}

macro_rules! if_some {
    ($ident: expr, ($var: ident) -> {$($tree:tt)*}) => {
        if let Some($var) = $ident {
            $($tree)*
        }
    };
}

impl ToTokens for PoolOptions {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        assert!(!(self.name.is_some() && self.name_fn.is_some()));

        if_some!(&self.threads, (w) -> {
            tokens.extend(quote!(.thread_number(#w)));
        });
        if_some!(&self.stack_size, (size) -> {
            tokens.extend(quote!(.thread_stack_size(#size)));
        });
        if_some!(&self.name, (name) -> {
            tokens.extend(quote!(.thread_name(#name)));
        });
        if_some!(&self.name_fn, (fun) -> {
            tokens.extend(quote!(.thread_name_fn(#fun)));
        });
        if_some!(&self.before, (fun) -> {
            tokens.extend(quote!(.before(#fun)));
        });
        if_some!(&self.after, (fun) -> {
            tokens.extend(quote!(.after(#fun)));
        });
    }
}
