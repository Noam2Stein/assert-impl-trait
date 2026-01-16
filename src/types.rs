use std::cmp::Ordering;

use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{ToTokens, quote_spanned};

/// Represents either a where-clause predicate:
///
/// ```ignore
/// SomeType: SomeTrait
/// ```
///
/// Or a predicate group which can define generic parameters and conditions:
///
/// ```ignore
/// for<T: Clone> where T: 'static {
///     T: Clone + 'static,
/// }
/// ```
#[derive(Debug, Clone)]
pub enum PredicateTree {
    Group(PredicateGroup),
    Predicate(Predicate),
}

/// Represents a delimited set of predicates like:
///
/// ```ignore
/// for<T: Clone> where T: 'static {
///     T: Clone + 'static,
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct PredicateGroup {
    /// The generic parameters declared using the `for` keyword.
    pub generic_params: Vec<GenericParam>,

    /// The predicates from the where-clause of the group.
    ///
    /// These are not the predicates that need to be asserted, these are the
    /// conditions for the asserts.
    pub where_predicates: Vec<Predicate>,

    /// The inner predicates that must be asserted to be true.
    ///
    /// These are predicate-trees meaning they can themselves be
    /// predicate-groups which allows for nesting.
    pub predicates: Vec<PredicateTree>,
}

/// Represents a where-clause predicate like `SomeType: SomeTrait`.
#[derive(Debug, Clone)]
pub struct Predicate {
    /// The `SomeType` in `SomeType: SomeTrait`.
    pub left_side: TokenStream,

    /// The `SomeTrait` in `SomeType: SomeTrait`.
    ///
    /// This does not include the colon.
    pub bound: TokenStream,

    pub span: Span,

    /// Is true if `left_side` is probably a dynamically-sized-type.
    ///
    /// This is used to hide a compiler limitation related
    /// dynamically-sized-types and the way `assert_impl` works.
    ///
    /// Currently this is only true for `dyn ...` left sides. Changing this
    /// would be a breaking change.
    pub unsized_left_side: bool,
}

/// Represents a generic parameter declaration like `const N: usize`.
#[derive(Debug, Clone)]
pub struct GenericParam {
    /// The parameter declaration.
    pub tokens: TokenStream,

    /// A generic argument with the name of the parameter.
    ///
    /// This corrosponds to the `N` in `const N: usize`.
    pub inline_arg: TokenStream,

    /// A type that depends on the generic parameter.
    ///
    /// This is used with `PhantomData` to use all generic parameters in struct
    /// declarations.
    pub marker_type: TokenStream,

    pub is_lifetime: bool,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub span: Span,
    pub message: String,
}

impl ToTokens for Predicate {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tk_colon = Punct::new(':', Spacing::Alone);

        self.left_side.to_tokens(tokens);
        tk_colon.to_tokens(tokens);
        self.bound.to_tokens(tokens);
    }
}

impl GenericParam {
    pub fn list_cmp(&self, other: &Self) -> Ordering {
        match (self.is_lifetime, other.is_lifetime) {
            (false, false) | (true, true) => Ordering::Equal,
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
        }
    }
}

impl ToTokens for GenericParam {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens);
    }
}

impl Error {
    pub fn new_at_span(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
}

impl ToTokens for Error {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let message = &self.message;
        tokens.extend(quote_spanned! { self.span => compile_error!(#message); });
    }
}
