use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree};
use quote::{TokenStreamExt, quote};

use crate::{
    token_iter::TokenIter,
    types::{Error, GenericParam, Predicate, PredicateGroup, PredicateTree},
};

pub fn parse_predicate_list(tokens: TokenStream, errors: &mut Vec<Error>) -> Vec<PredicateTree> {
    let tokens = &mut TokenIter::from(tokens);
    let mut result = Vec::new();

    while tokens.peek().is_some() {
        let predicate = match consume_predicate_tree(tokens, errors) {
            Ok(predicate) => predicate,
            Err(error) => {
                errors.push(error);
                break;
            }
        };

        if consume_optional_punct(tokens, ',').is_none()
            && let Some(token) = tokens.peek()
            && !matches!(predicate, PredicateTree::Group(_))
        {
            errors.push(Error::new_at_span(token.span(), "expected `,`"));
        }

        result.push(predicate);
    }

    result
}

////////////////////////////////////////////////////////////////////////////////
// Consume
////////////////////////////////////////////////////////////////////////////////

fn consume_predicate_tree(
    tokens: &mut TokenIter,
    errors: &mut Vec<Error>,
) -> Result<PredicateTree, Error> {
    if consume_optional_ident(tokens, "for").is_some() {
        consume_punct(tokens, '<')?;

        let mut generic_params = Vec::new();
        while tokens.peek().is_some() && !peek_punct(tokens, '>') {
            generic_params.push(consume_generic_param(tokens)?);

            if consume_optional_punct(tokens, ',').is_none() {
                break;
            }
        }

        consume_punct(tokens, '>')?;

        if let Some(i) = (1..generic_params.len())
            .find(|&i| generic_params[i].is_lifetime && !generic_params[i - 1].is_lifetime)
        {
            errors.push(Error::new_at_span(
                generic_params[i].span,
                "lifetime parameters must be declared prior to type and const parameters",
            ));
        }

        let where_predicates = consume_optional_where_clause(tokens)?.unwrap_or_default();

        let braces = consume_delimiter(tokens, Delimiter::Brace)?;
        let predicates = parse_predicate_list(braces.stream(), errors);

        return Ok(PredicateTree::Group(PredicateGroup {
            generic_params,
            where_predicates,
            predicates,
        }));
    }

    if let Some(where_predicates) = consume_optional_where_clause(tokens)? {
        let braces = consume_delimiter(tokens, Delimiter::Brace)?;
        let predicates = parse_predicate_list(braces.stream(), errors);

        return Ok(PredicateTree::Group(PredicateGroup {
            generic_params: Vec::new(),
            where_predicates,
            predicates,
        }));
    }

    Ok(PredicateTree::Predicate(consume_predicate(tokens)?))
}

fn consume_predicate(tokens: &mut TokenIter) -> Result<Predicate, Error> {
    let span = tokens.span();

    let unsized_left_side = peek_ident(tokens, "dyn");

    let left_side = consume_type_expr(tokens)?;
    let _ = consume_punct(tokens, ':')?;
    let bound = consume_optional_type_expr(tokens).unwrap_or_default();

    Ok(Predicate {
        left_side,
        bound,
        span,
        unsized_left_side,
    })
}

fn consume_generic_param(tokens: &mut TokenIter) -> Result<GenericParam, Error> {
    let mut result;

    if let Some(lifetime_prefix) = consume_optional_punct(tokens, '\'') {
        let lifetime_prefix_span = lifetime_prefix.span();
        let mut lifetime_prefix = Punct::new('\'', Spacing::Joint);
        lifetime_prefix.set_span(lifetime_prefix_span);

        let name = consume_any_ident(tokens)?;

        result = GenericParam {
            tokens: quote! { #lifetime_prefix #name },
            inline_arg: quote! { #lifetime_prefix #name },
            marker_type: quote! { &#lifetime_prefix #name () }.into_iter().collect(),
            is_lifetime: true,
            span: name.span(),
        };
    } else if let Some(const_prefix) = consume_optional_ident(tokens, "const") {
        let name = consume_any_ident(tokens)?;

        result = GenericParam {
            tokens: quote! { #const_prefix #name },
            inline_arg: quote! { #name },
            marker_type: quote! { () }.into_iter().collect(),
            is_lifetime: false,
            span: name.span(),
        };
    } else {
        let name = consume_any_ident(tokens)?;

        result = GenericParam {
            tokens: quote! { #name },
            inline_arg: quote! { #name },
            marker_type: quote! { #name },
            is_lifetime: false,
            span: name.span(),
        };
    }

    if let Some(bound_prefix) = consume_optional_punct(tokens, ':') {
        let bound = consume_type_expr(tokens)?;

        result.tokens.extend(quote! { #bound_prefix #bound });
    }

    Ok(result)
}

fn consume_type_expr(tokens: &mut TokenIter) -> Result<TokenStream, Error> {
    consume_optional_type_expr(tokens)
        .ok_or_else(|| Error::new_at_span(tokens.span(), "expected a type expression"))
}

fn consume_delimiter(tokens: &mut TokenIter, expected: Delimiter) -> Result<Group, Error> {
    consume_optional_delimiter(tokens, expected).ok_or_else(|| {
        let expected_str = match expected {
            Delimiter::Brace => "braces",
            Delimiter::Bracket => "brackets",
            Delimiter::Parenthesis => "parentheses",
            Delimiter::None => unreachable!(),
        };

        Error::new_at_span(tokens.span(), format!("expected {expected_str}"))
    })
}

fn consume_any_ident(tokens: &mut TokenIter) -> Result<Ident, Error> {
    consume_optional_any_ident(tokens)
        .ok_or_else(|| Error::new_at_span(tokens.span(), "expected an identifier"))
}

fn consume_punct(tokens: &mut TokenIter, expected: char) -> Result<Punct, Error> {
    consume_optional_punct(tokens, expected)
        .ok_or_else(|| Error::new_at_span(tokens.span(), format!("expected `{expected}`")))
}

////////////////////////////////////////////////////////////////////////////////
// Consume Optional
////////////////////////////////////////////////////////////////////////////////

fn consume_optional_where_clause(tokens: &mut TokenIter) -> Result<Option<Vec<Predicate>>, Error> {
    if consume_optional_ident(tokens, "where").is_none() {
        return Ok(None);
    }

    let mut result = Vec::new();

    while tokens.peek().is_some()
        && !peek_punct(tokens, ';')
        && !peek_delimiter(tokens, Delimiter::Brace)
    {
        result.push(consume_predicate(tokens)?);

        consume_optional_punct(tokens, ',');
    }

    Ok(Some(result))
}

fn consume_optional_type_expr(tokens: &mut TokenIter) -> Option<TokenStream> {
    let mut result = TokenStream::new();
    let mut depth = 0;

    while tokens.peek().is_some() {
        if depth <= 0 && peek_punct(tokens, ':') && peek2_punct(tokens, ':') {
            result.append(tokens.next().expect("peek ensures a token exists"));
            result.append(tokens.next().expect("a path seperator has two tokens"));
            continue;
        }

        if depth <= 0
            && (peek_punct(tokens, ',')
                || peek_punct(tokens, ';')
                || peek_punct(tokens, ':')
                || peek_punct(tokens, '>')
                || peek_punct(tokens, '=')
                || peek_delimiter(tokens, Delimiter::Brace))
        {
            break;
        }

        if peek_punct(tokens, '<') {
            depth += 1;
        } else if peek_punct(tokens, '>') {
            depth -= 1;
        }

        result.append(tokens.next().expect("peek ensures a token exists"));
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn consume_optional_delimiter(tokens: &mut TokenIter, expected: Delimiter) -> Option<Group> {
    if let Some(TokenTree::Group(token)) = tokens.peek()
        && token.delimiter() == expected
    {
        let result = token.clone();
        tokens.next();

        Some(result)
    } else {
        None
    }
}

fn consume_optional_ident(tokens: &mut TokenIter, expected: &str) -> Option<Ident> {
    if let Some(TokenTree::Ident(token)) = tokens.peek()
        && token == expected
    {
        let result = token.clone();
        tokens.next();

        Some(result)
    } else {
        None
    }
}

fn consume_optional_any_ident(tokens: &mut TokenIter) -> Option<Ident> {
    if let Some(TokenTree::Ident(token)) = tokens.peek() {
        let result = token.clone();
        tokens.next();

        Some(result)
    } else {
        None
    }
}

fn consume_optional_punct(tokens: &mut TokenIter, expected: char) -> Option<Punct> {
    if let Some(TokenTree::Punct(token)) = tokens.peek()
        && token.as_char() == expected
    {
        let result = token.clone();
        tokens.next();

        Some(result)
    } else {
        None
    }
}

////////////////////////////////////////////////////////////////////////////////
// Peek
////////////////////////////////////////////////////////////////////////////////

fn peek_delimiter(tokens: &TokenIter, expected: Delimiter) -> bool {
    if let Some(TokenTree::Group(token)) = tokens.peek() {
        token.delimiter() == expected
    } else {
        false
    }
}

fn peek_ident(tokens: &TokenIter, expected: &str) -> bool {
    if let Some(TokenTree::Ident(token)) = tokens.peek() {
        token == expected
    } else {
        false
    }
}

fn peek_punct(tokens: &TokenIter, expected: char) -> bool {
    if let Some(TokenTree::Punct(token)) = tokens.peek() {
        token.as_char() == expected
    } else {
        false
    }
}

fn peek2_punct(tokens: &TokenIter, expected: char) -> bool {
    if let Some(TokenTree::Punct(token)) = tokens.peek2() {
        token.as_char() == expected
    } else {
        false
    }
}
