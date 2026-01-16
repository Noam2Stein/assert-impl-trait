use proc_macro2::{Span, TokenStream, TokenTree};

#[derive(Debug)]
pub struct TokenIter {
    /// The tokens are in reverse order so that pop returns the next token.
    reversed_tokens: Vec<TokenTree>,

    next_span: Span,
}

impl TokenIter {
    pub fn next(&mut self) -> Option<TokenTree> {
        let result = self.reversed_tokens.pop();

        self.next_span = self
            .reversed_tokens
            .last()
            .map_or_else(Span::call_site, TokenTree::span);

        result
    }

    pub fn peek(&self) -> Option<&TokenTree> {
        self.reversed_tokens.last()
    }

    pub fn peek2(&self) -> Option<&TokenTree> {
        if self.reversed_tokens.len() < 2 {
            return None;
        }

        Some(&self.reversed_tokens[self.reversed_tokens.len() - 2])
    }

    pub fn span(&self) -> Span {
        self.next_span
    }
}

impl From<TokenStream> for TokenIter {
    fn from(value: TokenStream) -> Self {
        let reversed_tokens = value
            .into_iter()
            .collect::<Vec<TokenTree>>()
            .into_iter()
            .rev()
            .collect::<Vec<TokenTree>>();

        let next_span = reversed_tokens
            .last()
            .map_or_else(Span::call_site, TokenTree::span);

        Self {
            reversed_tokens,
            next_span,
        }
    }
}
