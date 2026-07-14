use anyhow::Context as _;
use std::fmt;

mod expression;
mod tokenizer;

pub use expression::Expression;

pub fn parse_expression(input: &str) -> anyhow::Result<Expression> {
    let tokens = input
        .parse()
        .inspect_err(|err| match err {
            tokenizer::ParseError::EndOfInput => error!("End of input"),
            tokenizer::ParseError::InvalidToken { position } => {
                error!("Invalid token at {position}");
            }
            tokenizer::ParseError::InvalidFloat { position } => {
                error!("Invalid float at {position}");
            }
            tokenizer::ParseError::InvalidInteger { position } => {
                error!("Invalid integer at {position}");
            }
        })
        .context("tokenize")?;
    let expression = Expression::parse(tokens)
        .inspect_err(|err| match err {
            expression::ParseError::EndOfInput => error!("End of input"),
            expression::ParseError::UnexpectedToken { span } => {
                error!("Unexpected token at {span}");
            }
        })
        .context("parse")?;
    Ok(expression)
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
