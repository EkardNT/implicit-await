extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree, Span};
use syn::{parse_macro_input, parse_quote, ItemFn, Type, AngleBracketedGenericArguments, PathArguments, Ident};
use syn::fold::Fold;

#[proc_macro_attribute]
pub fn implicit_await(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);
    let mut folder = ImplicitAwaitFold { defer: 0 };
    let transformed_fn = folder.fold_item_fn(item_fn);
    quote::quote!(#transformed_fn).into()
}

struct ImplicitAwaitFold {
    defer: usize
}

impl Fold for ImplicitAwaitFold {
    fn fold_expr(&mut self, expr: syn::Expr) -> syn::Expr {
        let subfolded = syn::fold::fold_expr(self, expr);
        match (self.defer, &subfolded) {
            (0, syn::Expr::Call(call)) => {
                parse_quote!({{ #call }.as_future().await})
            },
            (0, syn::Expr::MethodCall(method_call)) => {
                parse_quote!({{ #method_call }.as_future().await})
            },
            _ => subfolded
        }
    }

    fn fold_impl_item_method(&mut self, method: syn::ImplItemMethod) -> syn::ImplItemMethod {
        let mut subfolded = syn::fold::fold_impl_item_method(self, method);
        subfolded.block.stmts.insert(0, parse_quote!(use implicit_await::as_future::FutureAsFuture;));
        subfolded.block.stmts.insert(0, parse_quote!(use implicit_await::as_future::NonFutureAsFuture;));
        subfolded
    }

    fn fold_item_fn(&mut self, func: syn::ItemFn) -> syn::ItemFn {
        let mut subfolded = syn::fold::fold_item_fn(self, func);
        subfolded.block.stmts.insert(0, parse_quote!(use implicit_await::as_future::FutureAsFuture;));
        subfolded.block.stmts.insert(0, parse_quote!(use implicit_await::as_future::NonFutureAsFuture;));
        subfolded
    }

    fn fold_expr_macro(&mut self, mac: syn::ExprMacro) -> syn::ExprMacro {
        let is_defer = mac.mac.path.segments.iter()
            .last()
            .map_or(false, |segment| segment.ident.to_string() == "defer");
        if is_defer {
            self.defer += 1;
        }
        let subfolded = syn::fold::fold_expr_macro(self, mac);
        if is_defer {
            self.defer -= 1;
        }
        subfolded
    }
}

// Meant to be used by third-party crates outside of implicit-await.
#[proc_macro]
pub fn as_future(input: TokenStream) -> TokenStream {
    as_future_impl(TokenStream2::from(input), "implicit_await")
}

// Meant to be used by the implicit-await crate. Different from as_future due to lack of $crate support.
#[proc_macro]
pub fn as_future_internal(input: TokenStream) -> TokenStream {
    as_future_impl(TokenStream2::from(input), "crate")
}

// Note: want this to return a TokenStream2, but the parse_macro_input! macro seems to only work in
// functions which return a TokenStream, so I'll need to replace usage of that macro first.
fn as_future_impl(input: TokenStream2, crate_prefix: &'static str) -> TokenStream {
    let crate_prefix = Ident::new(crate_prefix, Span::call_site());
    let mut types: Vec<Type> = Vec::new();
    let mut current_type: TokenStream2 = TokenStream2::new();
    for token in input {
        if !is_delimiter(&token) {
            current_type.extend(vec![token.clone()].drain(..));
            continue;
        }
        let current_type_as_proc_macro_stream = TokenStream::from(current_type);
        let parsed_type: Type = parse_macro_input!(current_type_as_proc_macro_stream as Type);
        types.push(parsed_type);
        current_type = TokenStream2::new();
    }
    if !current_type.is_empty() {
        let current_type_as_proc_macro_stream = TokenStream::from(current_type);
        let parsed_type: Type = parse_macro_input!(current_type_as_proc_macro_stream as Type);
        types.push(parsed_type);
    }
    let mut output = TokenStream2::new();
    for impl_type in &types {
        let generic_args = get_generic_args(&impl_type);
        output.extend(quote::quote!(
            impl #generic_args #crate_prefix::as_future::NonFutureAsFuture for #impl_type {
                fn as_future(self) -> #crate_prefix::as_future::Ready<Self> {
                    #crate_prefix::as_future::ready(self)
                }
            }
        ));
    }
    output.into()
}

fn is_delimiter(token: &TokenTree) -> bool {
    match token {
        TokenTree::Punct(punct) => punct.as_char() == ',',
        _ => false
    }
}

fn get_generic_args(impl_type: &Type) -> Option<AngleBracketedGenericArguments> {
    if let Type::Path(type_path) = impl_type {
        type_path.path.segments.iter()
            .filter_map(|path_segment| match &path_segment.arguments {
                PathArguments::AngleBracketed(args) => Some(args),
                _ => None
            })
            .last()
            .cloned()
    } else {
        None
    }
}