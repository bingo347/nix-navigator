use crate::deserialize_wrapper::DeserializeWrappable;
use std::fmt;

mod content;
mod expression;
mod template;
mod tokenizer;

pub use self::{
    content::{FullTemplate, Locale, Rule},
    expression::Expression,
    template::Template,
};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Tokenize filed: {0}")]
    Tokenizer(#[from] tokenizer::ParseError),
    #[error("Parse expression filed: {0}")]
    Expression(#[from] expression::ParseError),
    #[error("Parse template filed: {0}")]
    Template(#[from] template::ParseError),
}

impl DeserializeWrappable for Expression {
    type SourceType = String;

    #[expect(refining_impl_trait)]
    fn from_source(source: String) -> Result<Self, ParseError> {
        let tokens = source.parse()?;
        let expression = Expression::parse(tokens)?;
        Ok(expression)
    }
}

impl DeserializeWrappable for Template {
    type SourceType = String;

    #[expect(refining_impl_trait)]
    fn from_source(source: String) -> Result<Self, ParseError> {
        Ok(source.parse()?)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    const fn with_end(mut self, end: usize) -> Self {
        self.end = end;
        self
    }

    fn join(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
