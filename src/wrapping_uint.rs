extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::fold::{self, Fold};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, parse_quote, Attribute, BinOp, Expr, ItemFn};

/// Dummy struct to use for token parsing.
#[derive(Debug)]
struct Args {}

impl Parse for Args {
    fn parse(_input: ParseStream) -> Result<Self> {
        Ok(Args {})
    }
}

impl Args {
    /// Produces an expression that is a binary operation.
    /// TODO: do we need attrs?
    fn binary_op(&mut self, _attrs: Vec<Attribute>, left: Expr, op: &BinOp, right: Expr) -> Expr {
        let left = match left {
            Expr::Binary(e) => self.binary_op(e.attrs, *e.left, &e.op, *e.right),
            _ => fold::fold_expr(self, left),
        };
        let right = match right {
            Expr::Binary(e) => self.binary_op(e.attrs, *e.left, &e.op, *e.right),
            _ => fold::fold_expr(self, right),
        };
        match op {
            BinOp::Add(_op) => {
                return parse_quote!(
                    #left.wrapping_add(#right)
                );
            }
            BinOp::Mul(_op) => {
                return parse_quote!(
                    #left.wrapping_mul(#right)
                );
            }
            _ => {
                return parse_quote!(
                    #left #op #right
                );
            }
        }
    }

    /// Produces an expression that is an assign operation.
    /// TODO: do we need attrs?
    fn assign_op(&mut self, _attrs: Vec<Attribute>, left: Expr, op: &BinOp, right: Expr) -> Expr {
        let left = match left {
            Expr::Binary(e) => self.binary_op(e.attrs, *e.left, &e.op, *e.right),
            _ => fold::fold_expr(self, left),
        };
        let right = match right {
            Expr::Binary(e) => self.binary_op(e.attrs, *e.left, &e.op, *e.right),
            _ => fold::fold_expr(self, right),
        };
        match op {
            BinOp::AddEq(_op) => {
                return parse_quote!(
                    #left = #left.wrapping_add(#right)
                );
            }
            BinOp::MulEq(_op) => {
                return parse_quote!(
                    #left = #left.wrapping_mul(#right)
                );
            }
            _ => {
                return parse_quote!(
                    #left #op #right
                );
            }
        }
    }
}

/// The `Fold` trait is a way to traverse an owned syntax tree and replace some
/// of its nodes.
///
/// Syn provides two other syntax tree traversal traits: `Visit` which walks a
/// shared borrow of a syntax tree, and `VisitMut` which walks an exclusive
/// borrow of a syntax tree and can mutate it in place.
///
/// All three traits have a method corresponding to each type of node in Syn's
/// syntax tree. All of these methods have default no-op implementations that
/// simply recurse on any child nodes. We can override only those methods for
/// which we want non-default behavior. In this case the traversal needs to
/// transform `Expr` nodes.
impl Fold for Args {
    fn fold_expr(&mut self, e: Expr) -> Expr {
        match e {
            Expr::Binary(e) => self.binary_op(e.attrs, *e.left, &e.op, *e.right),
            Expr::AssignOp(e) => self.assign_op(e.attrs, *e.left, &e.op, *e.right),
            _ => fold::fold_expr(self, e),
        }
    }
}

/// Make addition and multiplication wrapping in the annotated function.
///
/// # Example
///
/// ```rust,ignore
/// #[wrapit]
/// fn oops() -> bool {
///     let a: u32 = std::u32::MAX;
///     let b: u32 = 2;
///     let r = a + b;
///     r == 1
/// }
/// ```
#[proc_macro_attribute]
pub fn wrappit(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    // Get something to fold on.
    // TODO: is there a nicer way to do this?
    let mut args = parse_macro_input!(args as Args);

    // Use a syntax tree traversal to transform the function body.
    let output = args.fold_item_fn(input);

    // Hand the resulting function body back to the compiler.
    TokenStream::from(quote!(#output))
}
