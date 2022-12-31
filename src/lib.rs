extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    bracketed,
    ext::IdentExt,
    parse,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Pat, Result, Stmt, Token, UnOp,
};
struct BytePat {
    pub ident: Ident,
    pub _colon: Token![:],
    pub ty: syn::Path,
    pub _slash: Token![/],
    pub byte_order: Ident,
}

enum PatOrBytePat {
    Pat(Pat),
    BytePat(BytePat),
}

impl Parse for BytePat {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            _colon: input.parse()?,
            ty: input.parse()?,
            _slash: input.parse()?,
            byte_order: input.parse()?,
        })
    }
}

impl Parse for PatOrBytePat {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::Ident::peek_any) && input.peek2(Token![:]) {
            input.parse().map(PatOrBytePat::BytePat)
        } else {
            input.parse().map(PatOrBytePat::Pat)
        }
    }
}

struct Match {
    pub _bracket_token: token::Bracket,
    pub patterns: Punctuated<PatOrBytePat, Token![,]>,
    pub _eq: Token![=],
    pub subject: Expr,
}

impl Parse for Match {
    fn parse(input: ParseStream) -> Result<Self> {
        let bracketed_content;
        Ok(Match {
            _bracket_token: bracketed!(bracketed_content in input),
            patterns: Punctuated::parse_terminated(&bracketed_content).unwrap(),
            _eq: input.parse()?,
            subject: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn match_bytes(item: TokenStream) -> TokenStream {
    let mut conversions: Vec<Stmt> = vec![];
    let Match {
        subject, patterns, ..
    } = parse(item).unwrap();

    let mut expanded_byte_pats: Punctuated<Pat, Token![,]> = Punctuated::new();
    for pattern in patterns {
        match pattern {
            PatOrBytePat::Pat(pattern) => {
                expanded_byte_pats.push(pattern);
            }
            PatOrBytePat::BytePat(byte_pat) => {
                conversions.push(conversion_for(&byte_pat));

                for expanded_pattern in expand_byte_pat(byte_pat).iter().cloned() {
                    expanded_byte_pats.push(expanded_pattern);
                }
            }
        }
    }

    quote!(let [#expanded_byte_pats] = #subject;
      #(#conversions)*
    )
    .into()
}

fn conversion_for(byte_pat: &BytePat) -> Stmt {
    let mut elems = Punctuated::new();
    for position in 0..size_of(byte_pat.ty.clone()) {
        let mut segments = Punctuated::new();
        segments.push(syn::PathSegment {
            arguments: syn::PathArguments::None,
            ident: Ident::new(
                &(byte_pat.ident.to_string() + &position.to_string()),
                Span::call_site(),
            ),
        });
        elems.push(syn::Expr::Unary(syn::ExprUnary {
            attrs: vec![],
            op: UnOp::Deref(Token![*](Span::call_site())),
            expr: Box::new(syn::Expr::Path(syn::ExprPath {
                attrs: vec![],
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: segments,
                },
            })),
        }));
    }
    let mut segments = Punctuated::new();
    segments.push(syn::PathSegment {
        arguments: syn::PathArguments::None,
        ident: byte_pat.ty.get_ident().unwrap().clone(),
    });
    segments.push(syn::PathSegment {
        arguments: syn::PathArguments::None,
        ident: Ident::new(
            &format!("from_{}_bytes", byte_pat.byte_order),
            Span::call_site(),
        ),
    });
    let func = syn::Expr::Path(syn::ExprPath {
        attrs: vec![],
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: segments,
        },
    });

    let mut args = Punctuated::new();
    args.push(syn::Expr::Array(syn::ExprArray {
        attrs: vec![],
        bracket_token: syn::token::Bracket(Span::call_site()),
        elems: elems,
    }));
    let expr = syn::Expr::Call(syn::ExprCall {
        attrs: vec![],
        func: Box::new(func),
        paren_token: syn::token::Paren(Span::call_site()),
        args: args,
    });
    let pat: syn::Pat = syn::Pat::Ident(syn::PatIdent {
        attrs: vec![],
        subpat: None,
        mutability: None,
        ident: byte_pat.ident.clone(),
        by_ref: None,
    });
    Stmt::Semi(
        syn::Expr::Let(syn::ExprLet {
            let_token: Token![let](Span::call_site()),
            attrs: vec![],
            pat: pat,
            eq_token: Token![=](Span::call_site()),
            expr: Box::new(expr),
        }),
        syn::token::Semi(Span::call_site()),
    )
}

fn expand_byte_pat(byte_pat: BytePat) -> Vec<syn::Pat> {
    let mut result = vec![];
    for position in 0..size_of(byte_pat.ty) {
        let new_ident = Ident::new(
            &(byte_pat.ident.to_string().to_owned() + &position.to_string()),
            Span::call_site(),
        );
        result.push(Pat::Ident(syn::PatIdent {
            ident: new_ident,
            attrs: vec![],
            mutability: None,
            by_ref: None,
            subpat: None,
        }))
    }
    result
}

macro_rules! size_of_conditional {
    ($subject:expr, [$ty:ty]) => (
      if $subject.is_ident(stringify!($ty)) {
        return std::mem::size_of::<$ty>();
      } else {
       panic!("Unknown type: {}", stringify!($subject))
     });
    ($subject:expr, [$ty:ty, $($rest:ty),+]) => (
      if $subject.is_ident(stringify!($ty)) {
        return std::mem::size_of::<$ty>();
      } else {
        size_of_conditional!($subject, [$($rest),+])
    })
}

fn size_of(ty: syn::Path) -> usize {
    size_of_conditional!(
        ty,
        [u8, i8, u16, i16, u32, i32, u64, i64, i128, u128, isize, usize, f32, f64]
    )
}
