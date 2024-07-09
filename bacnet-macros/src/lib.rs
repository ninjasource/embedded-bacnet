use core::borrow::BorrowMut;
use proc_macro::TokenStream;
use quote::quote;
use syn::{fold::Fold, parse_macro_input, FnArg, ItemFn, Type};

struct Args {}

impl Fold for Args {
    fn fold_fn_arg(&mut self, mut i: syn::FnArg) -> syn::FnArg {
        match i.borrow_mut() {
            FnArg::Typed(x) => {
                match x.ty.as_mut() {
                    Type::Reference(r) => {
                        // remove the lifetime if there is one
                        r.lifetime = None;
                    }
                    _ => {
                        // do nothing
                    }
                }
            }
            _ => {
                // do nothing
            }
        }

        syn::fold::fold_fn_arg(self, i)
    }
}

/// This proc macro targets a function and will remove the lifetimes of any reference arguments
/// For example
///     fn foo(buf: &'a [u8])
///     will turn into
///     fn foo(buf: &[u8])
///
#[proc_macro_attribute]
pub fn remove_lifetimes_from_fn_args(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let mut args = Args {};

    // Use a syntax tree traversal to transform the function body.
    let output = args.fold_item_fn(input);

    // Hand the resulting function body back to the compiler.
    TokenStream::from(quote!(#output))
}
