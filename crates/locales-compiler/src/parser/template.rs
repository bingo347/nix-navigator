use super::Expression;
use std::str::FromStr;

pub struct Template {
    pub strings: Vec<String>,
    pub substitutions: Vec<Expression>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Tokenize filed: {0}")]
    Tokenizer(#[from] super::tokenizer::ParseError),
    #[error("Parse expression filed: {0}")]
    Expression(#[from] super::expression::ParseError),
    #[error("Uncompleted substitution")]
    UncompletedSubstitution,
}

enum State {
    String,
    Substitution,
}

impl FromStr for Template {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let mut strings = Vec::new();
        let mut substitutions = Vec::new();
        let mut current = String::new();
        let mut state = State::String;

        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            match state {
                State::String => {
                    if c == '{' && chars.peek() == Some(&'{') {
                        current.push(c);
                        chars.next();
                    } else if c == '{' {
                        strings.push(current);
                        current = String::new();
                        state = State::Substitution;
                    } else {
                        current.push(c);
                    }
                }
                State::Substitution => {
                    if c == '}' {
                        let tokens = current.parse()?;
                        let expression = Expression::parse(tokens)?;
                        substitutions.push(expression);
                        current = String::new();
                        state = State::String;
                    } else {
                        current.push(c);
                    }
                }
            }
        }

        if matches!(state, State::Substitution) {
            Err(ParseError::UncompletedSubstitution)
        } else {
            strings.push(current);
            Ok(Self {
                strings,
                substitutions,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(s: &str) -> Template {
        s.parse().unwrap()
    }

    #[test]
    fn simple() {
        let template = template("Hello world");
        assert_eq!(template.strings, ["Hello world"]);
        assert!(template.substitutions.is_empty());
    }

    #[test]
    fn escape() {
        let template = template("Hello {{ world }");
        assert_eq!(template.strings, ["Hello { world }"]);
        assert!(template.substitutions.is_empty());
    }

    #[test]
    fn substitutions() {
        let span = super::super::Span::default();
        let template = template("{a} Hello {b} world {c}");
        assert_eq!(template.strings, ["", " Hello ", " world ", ""]);
        assert_eq!(
            template.substitutions,
            [
                Expression::Variable("a".into(), span),
                Expression::Variable("b".into(), span),
                Expression::Variable("c".into(), span)
            ]
        );
    }
}
