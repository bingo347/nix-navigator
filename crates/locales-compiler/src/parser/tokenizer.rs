use super::Span;
use std::{fmt, iter::Peekable, str::FromStr, sync::Arc};

#[derive(Debug, Clone, PartialEq)]
pub struct TokenStream {
    tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct Token {
    inner: TokenInner,
    span: Span,
}

#[derive(Debug, Clone)]
pub struct Literal {
    inner: LiteralInner,
    span: Span,
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct Punctuation {
    inner: PunctuationInner,
    span: Span,
}

#[derive(Debug, Clone, Eq)]
pub struct Identifier {
    inner: IdentifierInner,
    span: Span,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenInner {
    Literal(LiteralInner),
    Punctuation(PunctuationInner),
    Identifier(IdentifierInner),
}

#[derive(Debug, Clone, PartialEq)]
enum LiteralInner {
    String(Arc<str>),
    Float(f64),
    Integer(i64),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PunctuationInner {
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equals,
    NotEquals,
    Leather,
    Greater,
    LeatherEquals,
    GreaterEquals,
    Exclamation,
    DoubleAmpersand,
    DoublePipe,
    Comma,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
struct IdentifierInner {
    inner: Arc<str>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("End of input")]
    EndOfInput,
    #[error("Invalid token at position {position}")]
    InvalidToken { position: usize },
    #[error("Invalid float at position {position}")]
    InvalidFloat { position: usize },
    #[error("Invalid integer at position {position}")]
    InvalidInteger { position: usize },
}

impl TokenStream {
    const fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    pub const fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    #[cfg(test)]
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }
}

impl Token {
    const fn new(inner: TokenInner, span: Span) -> Self {
        Self { inner, span }
    }

    pub fn literal(&self) -> Option<Literal> {
        if let TokenInner::Literal(literal) = &self.inner {
            Some(Literal::new(literal.clone(), self.span))
        } else {
            None
        }
    }

    pub const fn punctuation(&self) -> Option<Punctuation> {
        if let TokenInner::Punctuation(punctuation) = &self.inner {
            Some(Punctuation::new(*punctuation, self.span))
        } else {
            None
        }
    }

    pub fn identifier(&self) -> Option<Identifier> {
        if let TokenInner::Identifier(identifier) = &self.inner {
            Some(Identifier::new(Arc::clone(&identifier.inner), self.span))
        } else {
            None
        }
    }
}

impl Literal {
    const fn new(inner: LiteralInner, span: Span) -> Self {
        Self { inner, span }
    }

    fn new_string(value: impl Into<Arc<str>>, span: Span) -> Self {
        Self::new(LiteralInner::String(value.into()), span)
    }

    const fn new_float(value: f64, span: Span) -> Self {
        Self::new(LiteralInner::Float(value), span)
    }

    #[cfg(test)]
    const fn new_integer(value: i64, span: Span) -> Self {
        Self::new(LiteralInner::Integer(value), span)
    }

    const fn new_boolean(value: bool, span: Span) -> Self {
        Self::new(LiteralInner::Boolean(value), span)
    }

    pub fn as_string(&self) -> Option<Arc<str>> {
        if let LiteralInner::String(value) = &self.inner {
            Some(Arc::clone(value))
        } else {
            None
        }
    }

    pub const fn as_float(&self) -> Option<f64> {
        if let LiteralInner::Float(value) = &self.inner {
            Some(*value)
        } else {
            None
        }
    }

    pub const fn as_integer(&self) -> Option<i64> {
        if let LiteralInner::Integer(value) = &self.inner {
            Some(*value)
        } else {
            None
        }
    }

    pub const fn as_boolean(&self) -> Option<bool> {
        if let LiteralInner::Boolean(value) = &self.inner {
            Some(*value)
        } else {
            None
        }
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

impl Punctuation {
    const fn new(inner: PunctuationInner, span: Span) -> Self {
        Self { inner, span }
    }

    const fn from_parser(span: Span, first: char, second: Option<char>) -> Option<(Self, bool)> {
        Some(match (first, second) {
            ('&', Some('&')) => (
                Self::new(
                    PunctuationInner::DoubleAmpersand,
                    span.with_end(span.start + 1),
                ),
                true,
            ),
            ('|', Some('|')) => (
                Self::new(PunctuationInner::DoublePipe, span.with_end(span.start + 1)),
                true,
            ),
            ('=', Some('=')) => (
                Self::new(PunctuationInner::Equals, span.with_end(span.start + 1)),
                true,
            ),
            ('!', Some('=')) => (
                Self::new(PunctuationInner::NotEquals, span.with_end(span.start + 1)),
                true,
            ),
            ('<', Some('=')) => (
                Self::new(
                    PunctuationInner::LeatherEquals,
                    span.with_end(span.start + 1),
                ),
                true,
            ),
            ('>', Some('=')) => (
                Self::new(
                    PunctuationInner::GreaterEquals,
                    span.with_end(span.start + 1),
                ),
                true,
            ),
            ('(', _) => (Self::new(PunctuationInner::LeftParen, span), false),
            (')', _) => (Self::new(PunctuationInner::RightParen, span), false),
            ('+', _) => (Self::new(PunctuationInner::Plus, span), false),
            ('-', _) => (Self::new(PunctuationInner::Minus, span), false),
            ('*', _) => (Self::new(PunctuationInner::Star, span), false),
            ('/', _) => (Self::new(PunctuationInner::Slash, span), false),
            ('%', _) => (Self::new(PunctuationInner::Percent, span), false),
            ('<', _) => (Self::new(PunctuationInner::Leather, span), false),
            ('>', _) => (Self::new(PunctuationInner::Greater, span), false),
            ('!', _) => (Self::new(PunctuationInner::Exclamation, span), false),
            (',', _) => (Self::new(PunctuationInner::Comma, span), false),
            _ => return None,
        })
    }

    pub const fn as_str(&self) -> &'static str {
        use PunctuationInner as P;
        match self.inner {
            P::LeftParen => "(",
            P::RightParen => ")",
            P::Plus => "+",
            P::Minus => "-",
            P::Star => "*",
            P::Slash => "/",
            P::Percent => "%",
            P::Equals => "==",
            P::NotEquals => "!=",
            P::Leather => "<",
            P::Greater => ">",
            P::LeatherEquals => "<=",
            P::GreaterEquals => ">=",
            P::Exclamation => "!",
            P::DoubleAmpersand => "&&",
            P::DoublePipe => "||",
            P::Comma => ",",
        }
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

impl Identifier {
    fn new(inner: impl Into<Arc<str>>, span: Span) -> Self {
        let inner = IdentifierInner {
            inner: inner.into(),
        };
        Self { inner, span }
    }

    pub fn as_inner(&self) -> Arc<str> {
        Arc::clone(&self.inner.inner)
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

impl FromStr for TokenStream {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let mut tokens = Vec::<Token>::new();
        let mut chars = s.char_indices().peekable();

        while let Some((start, current)) = chars.next() {
            if current.is_whitespace() {
                continue;
            }
            let span = Span::new(start, start);
            let next = chars.peek().map(|(_, c)| *c);

            if matches!(current, 'a'..='z' | 'A'..='Z' | '_') {
                let mut identifier = String::with_capacity(128);
                identifier.push(current);
                while let Some((_, c @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '_'))) = chars.peek() {
                    identifier.push(*c);
                    chars.next();
                }
                let span = span.with_end(start + identifier.len() - 1);
                tokens.push(match identifier.as_str() {
                    "true" => Literal::new_boolean(true, span).into(),
                    "false" => Literal::new_boolean(false, span).into(),
                    "NaN" => Literal::new_float(f64::NAN, span).into(),
                    _ => Identifier::new(identifier, span).into(),
                });
                continue;
            }

            if current.is_ascii_digit() {
                let (len, literal) = parse_number_literal(start, current, &mut chars)?;
                let span = span.with_end(start + len);
                tokens.push(Literal::new(literal, span).into());
                continue;
            }

            if current == '-'
                && let Some('0'..='9') = next
            {
                let (len, literal) = parse_number_literal(start, current, &mut chars)?;
                let span = span.with_end(start + len - 1);
                tokens.push(Literal::new(literal, span).into());
                continue;
            }

            if current == '\'' || current == '"' {
                let string = parse_string_literal(current, &mut chars)?;
                let span = span.with_end(start + string.len());
                tokens.push(Literal::new_string(string, span).into());
                continue;
            }

            if current == '∞' {
                let span = span.with_end(start + 3);
                tokens.push(Literal::new_float(f64::INFINITY, span).into());
                continue;
            }
            if current == '-' && next == Some('∞') {
                let span = span.with_end(start + 4);
                tokens.push(Literal::new_float(f64::NEG_INFINITY, span).into());
                chars.next();
                continue;
            }

            if let Some((punctuation, skip_next)) = Punctuation::from_parser(span, current, next) {
                tokens.push(punctuation.into());
                if skip_next {
                    chars.next();
                }
                continue;
            }

            return Err(ParseError::InvalidToken { position: start });
        }

        Ok(Self::new(tokens))
    }
}

impl From<Literal> for Token {
    fn from(literal: Literal) -> Self {
        Self::new(TokenInner::Literal(literal.inner), literal.span)
    }
}

impl From<Punctuation> for Token {
    fn from(punctuation: Punctuation) -> Self {
        Self::new(TokenInner::Punctuation(punctuation.inner), punctuation.span)
    }
}

impl From<Identifier> for Token {
    fn from(ident: Identifier) -> Self {
        Self::new(TokenInner::Identifier(ident.inner), ident.span)
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialEq for Punctuation {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            LiteralInner::String(value) => write!(f, "\"{value}\""),
            LiteralInner::Float(value) => write!(f, "{value}"),
            LiteralInner::Integer(value) => write!(f, "{value}"),
            LiteralInner::Boolean(value) => write!(f, "{value}"),
        }
    }
}

fn parse_number_literal(
    position: usize,
    first_char: char,
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<(usize, LiteralInner), ParseError> {
    let mut number = String::with_capacity(128);
    number.push(first_char);

    let mut dot_used = false;
    let mut exponent_used = false;
    while let Some((_, next_char)) = chars.peek() {
        match *next_char {
            '.' => {
                if dot_used {
                    return Err(ParseError::InvalidFloat { position });
                }
                dot_used = true;
                number.push('.');
                chars.next();
            }
            'e' | 'E' => {
                if exponent_used {
                    return Err(ParseError::InvalidFloat { position });
                }
                exponent_used = true;
                number.push('e');
                chars.next();

                if let Some((_, c @ ('+' | '-'))) = chars.peek() {
                    number.push(*c);
                    chars.next();
                }
            }
            '_' => {
                chars.next();
            }
            next_char if next_char.is_ascii_digit() => {
                number.push(next_char);
                chars.next();
            }
            'a'..='z' | 'A'..='Z' => {
                return Err(if dot_used || exponent_used {
                    ParseError::InvalidFloat { position }
                } else {
                    ParseError::InvalidInteger { position }
                });
            }
            _ => break,
        }
    }

    Ok((
        number.len(),
        if dot_used || exponent_used {
            LiteralInner::Float(
                number
                    .parse()
                    .map_err(|_| ParseError::InvalidFloat { position })?,
            )
        } else {
            LiteralInner::Integer(
                number
                    .parse()
                    .map_err(|_| ParseError::InvalidInteger { position })?,
            )
        },
    ))
}

fn parse_string_literal(
    opener: char,
    chars: &mut impl Iterator<Item = (usize, char)>,
) -> Result<String, ParseError> {
    let mut prev_is_escape = false;
    let mut is_closed = false;
    let mut result = String::new();
    for (_, c) in chars {
        match c {
            c if c == opener && !prev_is_escape => {
                is_closed = true;
                break;
            }
            '\\' => {
                prev_is_escape = !prev_is_escape;
                result.push(c);
            }
            _ => {
                prev_is_escape = false;
                result.push(c);
            }
        }
    }

    if !is_closed {
        return Err(ParseError::EndOfInput);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("123", &Literal::new_integer(123, Span::default()).into(); "int")]
    #[test_case("-123", &Literal::new_integer(-123, Span::default()).into(); "negative int")]
    #[test_case("123.456", &Literal::new_float(123.456, Span::default()).into(); "float")]
    #[test_case("-123.456", &Literal::new_float(-123.456, Span::default()).into(); "negative float")]
    #[test_case("1.234567e-2", &Literal::new_float(1.234_567e-2, Span::default()).into(); "float with e")]
    #[test_case("∞", &Literal::new_float(f64::INFINITY, Span::default()).into(); "infinity")]
    #[test_case("-∞", &Literal::new_float(f64::NEG_INFINITY, Span::default()).into(); "negative infinity")]
    #[test_case("(", &Punctuation::new(PunctuationInner::LeftParen, Span::default()).into(); "left paren")]
    #[test_case(")", &Punctuation::new(PunctuationInner::RightParen, Span::default()).into(); "right paren")]
    #[test_case("+", &Punctuation::new(PunctuationInner::Plus, Span::default()).into(); "plus")]
    #[test_case("-", &Punctuation::new(PunctuationInner::Minus, Span::default()).into(); "minus")]
    #[test_case("*", &Punctuation::new(PunctuationInner::Star, Span::default()).into(); "star")]
    #[test_case("/", &Punctuation::new(PunctuationInner::Slash, Span::default()).into(); "slash")]
    #[test_case("%", &Punctuation::new(PunctuationInner::Percent, Span::default()).into(); "percent")]
    #[test_case("==", &Punctuation::new(PunctuationInner::Equals, Span::default()).into(); "equals")]
    #[test_case("!=", &Punctuation::new(PunctuationInner::NotEquals, Span::default()).into(); "not equals")]
    #[test_case("<", &Punctuation::new(PunctuationInner::Leather, Span::default()).into(); "less")]
    #[test_case(">", &Punctuation::new(PunctuationInner::Greater, Span::default()).into(); "greater")]
    #[test_case("<=", &Punctuation::new(PunctuationInner::LeatherEquals, Span::default()).into(); "less equals")]
    #[test_case(">=", &Punctuation::new(PunctuationInner::GreaterEquals, Span::default()).into(); "greater equals")]
    #[test_case("!", &Punctuation::new(PunctuationInner::Exclamation, Span::default()).into(); "exclamation")]
    #[test_case("&&", &Punctuation::new(PunctuationInner::DoubleAmpersand, Span::default()).into(); "double ampersand")]
    #[test_case("||", &Punctuation::new(PunctuationInner::DoublePipe, Span::default()).into(); "double pipe")]
    #[test_case(",", &Punctuation::new(PunctuationInner::Comma, Span::default()).into(); "comma")]
    #[test_case("variable", &Identifier::new("variable", Span::default()).into(); "identifier")]
    fn parse_one_literal(s: &str, expected_token: &Token) {
        let tokens: TokenStream = s.parse().unwrap();
        assert_eq!(tokens.tokens().len(), 1);
        assert_eq!(tokens.tokens()[0], *expected_token);
    }

    #[test]
    fn parse_string_literal() {
        let tokens: TokenStream = r#""hello world""#.parse().unwrap();
        assert_eq!(tokens.tokens().len(), 1);
        assert_eq!(
            tokens.tokens()[0],
            Literal::new_string("hello world", Span::default()).into()
        );

        let tokens: TokenStream = "'hello world'".parse().unwrap();
        assert_eq!(tokens.tokens().len(), 1);
        assert_eq!(
            tokens.tokens()[0],
            Literal::new_string("hello world", Span::default()).into()
        );

        let tokens: TokenStream = "'hello\\'world'".parse().unwrap();
        assert_eq!(tokens.tokens().len(), 1);
        assert_eq!(
            tokens.tokens()[0],
            Literal::new_string("hello\\'world", Span::default()).into()
        );

        let tokens: TokenStream = r#"
            "hello world"
            'hello world'
            "hello\"world"
            'hello\'world'
            "#
        .parse()
        .unwrap();

        assert_eq!(
            tokens.tokens(),
            &[
                Literal::new_string("hello world", Span::default()).into(),
                Literal::new_string("hello world", Span::default()).into(),
                Literal::new_string("hello\\\"world", Span::default()).into(),
                Literal::new_string("hello\\'world", Span::default()).into(),
            ]
        );
    }

    #[test]
    fn parse_words() {
        let tokens: TokenStream = "NaN variable true false Some_Name_123".parse().unwrap();
        let literal = tokens.tokens()[0].literal().unwrap();
        assert!(matches!(literal.inner, LiteralInner::Float(f) if f.is_nan()));

        assert_eq!(
            &tokens.tokens()[1..],
            &[
                Identifier::new("variable", Span::default()).into(),
                Literal::new_boolean(true, Span::default()).into(),
                Literal::new_boolean(false, Span::default()).into(),
                Identifier::new("Some_Name_123", Span::default()).into(),
            ]
        );
    }

    #[test_case("123a"; "invalid number")]
    #[test_case("&"; "invalid punctuation")]
    #[test_case("'123"; "invalid string")]
    fn parse_invalids(s: &str) {
        assert!(s.parse::<TokenStream>().is_err());
    }

    #[test]
    fn parse_empty() {
        let tokens: TokenStream = "".parse().unwrap();
        assert_eq!(tokens.tokens().len(), 0);
    }

    #[test]
    fn parse_real_expressions() {
        let expressions = [
            "number % 10 == 1 && number % 100 != 11",
            "number % 10 >= 2 && number % 10 <= 4 && (number % 100 < 10 || number % 100 >= 20)",
            "number == 1",
        ];
        for expr in expressions {
            assert!(expr.parse::<TokenStream>().is_ok());
        }
    }
}
