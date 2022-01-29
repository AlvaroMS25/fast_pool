mod options;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Result, ItemFn, AttributeArgs, parse2, parse_macro_input};
use darling::FromMeta;

/// Creates and initializes a new thread pool.
///
/// This macro accepts several inputs to modify the thread pool.
///
/// Available inputs:
///
/// - threads: sets the number of threads the pool will use.
/// - stack_size: sets the size of the stack memory for worker threads.
/// - name: sets the name of worker threads.
/// - name_fn: sets the function used to determine the name of worker threads.
/// - before: sets a function to execute before every task.
/// - after: sets a function to execute after every task.
///
/// # Examples
///
/// ## Using before & after hooks:
/// ```rust,ignore
/// fn hook() {
///     println!("Hook triggered!");
/// }
/// #[fast_pool::init(before = "hook", after = "hook")]
/// fn main() {
///     let handle = fast_pool::spawn(|| {
///         println!("Hello world");
///         ":D"
///     });
///     println!("{}", handle.wait().unwrap());
/// }
/// ```
///
/// ## Setting a fixed name for workers:
/// ```rust,ignore
/// #[fast_pool::init(name = "Worker")]
/// fn main() {}
/// ```
///
/// ## Using a function to provide thread names:
/// ```rust,ignore
/// fn name() -> String {
///     String::from("Worker");
/// }
/// #[fast_pool::init(name_fn = "name")]
/// fn main() {}
/// ```
///
/// ## Setting thread number & stack size:
/// ```rust,ignore
/// #[fast_pool::init(threads = 4, stack_size = 4096)]
/// fn main() {}
/// ```
#[proc_macro_attribute]
pub fn init(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    extract_output(_init(args, input.into()))
}

fn _init(args: AttributeArgs, input: TokenStream2) -> Result<TokenStream2> {
    let mut fun = parse2::<ItemFn>(input)?;

    let options = options::PoolOptions::from_list(&args).unwrap();

    let builder = quote::quote! {
        fast_pool::ThreadPoolBuilder::new()
            #options
            .build()
            .expect("Failed to build thread pool");
    };

    let block = &fun.block;
    *fun.block = parse2(quote::quote! {{
        #builder
        #block
    }})?;

    Ok(quote::quote!(#fun))
}

fn extract_output(res: Result<TokenStream2>) -> TokenStream {
    match res {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error()
    }.into()
}
