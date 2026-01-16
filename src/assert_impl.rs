use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};

use crate::{
    parse::parse_predicate_list,
    types::{GenericParam, Predicate, PredicateTree},
};

pub fn main(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut result = TokenStream::new();
    let mut errors = Vec::new();

    push_trees(
        parse_predicate_list(tokens.into(), &mut errors),
        Vec::new(),
        &mut Vec::new(),
        &mut result,
    );

    for error in errors {
        error.to_tokens(&mut result);
    }

    result.into()
}

fn push_trees(
    trees: Vec<PredicateTree>,
    mut generic_params: Vec<&GenericParam>,
    where_predicates: &mut Vec<Predicate>,
    tokens: &mut TokenStream,
) {
    let predicates = push_groups(trees, &generic_params, where_predicates, tokens);

    generic_params.sort_by(|a, b| a.list_cmp(b));

    let inline_generic_args = generic_params
        .iter()
        .map(|p| &p.inline_arg)
        .collect::<Vec<_>>();

    let generic_marker_types = generic_params
        .iter()
        .map(|p| &p.marker_type)
        .collect::<Vec<_>>();

    let mut context_tokens = TokenStream::new();

    for predicate in predicates {
        let left_side = predicate.left_side;
        let bound = predicate.bound;

        let optional_unsized = if predicate.unsized_left_side {
            quote! { ?Sized + }
        } else {
            TokenStream::new()
        };

        context_tokens.extend(quote_spanned! {
            predicate.span => {
                _HelperTy::<#(#inline_generic_args,)* #left_side>(
                    #(
                        ::core::marker::PhantomData::<#generic_marker_types>,
                    )*
                    ::core::marker::PhantomData,
                );

                struct _HelperTy<#(#generic_params,)* _AssertTy: #optional_unsized #bound>(
                    #(
                        ::core::marker::PhantomData<#generic_marker_types>,
                    )*
                    ::core::marker::PhantomData<_AssertTy>,
                )
                where
                    #(#where_predicates),*;
            }
        });
    }

    tokens.extend(quote! {
        #[allow(clippy::all)]
        const _: () = {
            fn _context<#(#generic_params),*>() where #(#where_predicates),* {
                #context_tokens
            }
        };
    });
}

fn push_groups(
    trees: Vec<PredicateTree>,
    generic_params: &Vec<&GenericParam>,
    where_predicates: &mut Vec<Predicate>,
    tokens: &mut TokenStream,
) -> Vec<Predicate> {
    let mut predicates = Vec::with_capacity(trees.len());

    for tree in trees {
        let group = match tree {
            PredicateTree::Group(group) => group,
            PredicateTree::Predicate(predicate) => {
                predicates.push(predicate);
                continue;
            }
        };

        let generic_params = generic_params
            .iter()
            .copied()
            .chain(&group.generic_params)
            .collect();

        let original_where_predicate_count = where_predicates.len();
        where_predicates.extend(group.where_predicates);

        push_trees(group.predicates, generic_params, where_predicates, tokens);

        where_predicates.truncate(original_where_predicate_count);
    }

    predicates
}
