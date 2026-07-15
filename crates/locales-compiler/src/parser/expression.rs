use super::{
    Span,
    tokenizer::{Punctuation, TokenStream},
};
use std::{collections::VecDeque, sync::Arc};

#[derive(Debug, Clone)]
pub enum Expression {
    Call(CallExpression, Span),
    Variable(Arc<str>, Span),
    String(Arc<str>, Span),
    Float(f64, Span),
    Int(i64, Span),
    Bool(bool, Span),
    Not(Box<Expression>, Span),
    Minus(Box<Expression>, Span),
    And(Box<Expression>, Box<Expression>, Span),
    Or(Box<Expression>, Box<Expression>, Span),
    Eq(Box<Expression>, Box<Expression>, Span),
    Ne(Box<Expression>, Box<Expression>, Span),
    Gt(Box<Expression>, Box<Expression>, Span),
    Ge(Box<Expression>, Box<Expression>, Span),
    Lt(Box<Expression>, Box<Expression>, Span),
    Le(Box<Expression>, Box<Expression>, Span),
    Add(Box<Expression>, Box<Expression>, Span),
    Sub(Box<Expression>, Box<Expression>, Span),
    Mul(Box<Expression>, Box<Expression>, Span),
    Div(Box<Expression>, Box<Expression>, Span),
    Mod(Box<Expression>, Box<Expression>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpression {
    pub callee: Arc<str>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("End of input")]
    EndOfInput,
    #[error("Unexpected token at {span}")]
    UnexpectedToken { span: Span },
}

#[derive(Debug, Clone)]
enum Tree {
    Expression(Expression),
    Punctuation(Punctuation),
    Nested(Vec<Tree>, Span),
}

#[derive(Debug, Clone)]
enum Item {
    Expression(Expression),
    Punctuation(Punctuation),
}

impl Expression {
    pub fn parse(tokens: TokenStream) -> Result<Self, ParseError> {
        if tokens.is_empty() {
            return Err(ParseError::EndOfInput);
        }

        let mut stack = Vec::new();
        let mut current = Vec::new();
        for token in tokens.into_tokens() {
            if let Some(punct) = token.punctuation() {
                match punct.as_str() {
                    "(" => {
                        stack.push((current, punct.span()));
                        current = Vec::new();
                    }
                    ")" => {
                        let Some((previous, span)) = stack.pop() else {
                            return Err(ParseError::UnexpectedToken { span: punct.span() });
                        };
                        let nested = current;
                        current = previous;
                        current.push(Tree::Nested(nested, span.join(punct.span())));
                    }
                    _ => {
                        current.push(Tree::Punctuation(punct));
                    }
                }
            } else if let Some(ident) = token.identifier() {
                current.push(Tree::Expression(Self::Variable(
                    ident.as_inner(),
                    ident.span(),
                )));
            } else if let Some(lit) = token.literal() {
                let span = lit.span();
                let s = lit.as_string();
                let i = lit.as_integer();
                let f = lit.as_float();
                let b = lit.as_boolean();

                let expr = match (s, i, f, b) {
                    (Some(s), None, None, None) => Self::String(s, span),
                    (None, Some(i), None, None) => Self::Int(i, span),
                    (None, None, Some(f), None) => Self::Float(f, span),
                    (None, None, None, Some(b)) => Self::Bool(b, span),
                    _ => unreachable!(),
                };
                current.push(Tree::Expression(expr));
            }
        }

        if !stack.is_empty() {
            return Err(ParseError::EndOfInput);
        }

        Self::reduce_tree(current)
    }

    pub const fn span(&self) -> Span {
        match self {
            Self::Call(_, span)
            | Self::Variable(_, span)
            | Self::String(_, span)
            | Self::Float(_, span)
            | Self::Int(_, span)
            | Self::Bool(_, span)
            | Self::Not(_, span)
            | Self::Minus(_, span)
            | Self::And(_, _, span)
            | Self::Or(_, _, span)
            | Self::Eq(_, _, span)
            | Self::Ne(_, _, span)
            | Self::Gt(_, _, span)
            | Self::Ge(_, _, span)
            | Self::Lt(_, _, span)
            | Self::Le(_, _, span)
            | Self::Add(_, _, span)
            | Self::Sub(_, _, span)
            | Self::Mul(_, _, span)
            | Self::Div(_, _, span)
            | Self::Mod(_, _, span) => *span,
        }
    }

    fn reduce_tree(tree: Vec<Tree>) -> Result<Self, ParseError> {
        if tree.is_empty() {
            return Err(ParseError::EndOfInput);
        }
        if tree.len() == 1 {
            return match tree.into_iter().next() {
                Some(Tree::Expression(expr)) => Ok(expr),
                Some(Tree::Punctuation(punct)) => {
                    Err(ParseError::UnexpectedToken { span: punct.span() })
                }
                Some(Tree::Nested(nested, _)) => Self::reduce_tree(nested),
                None => unreachable!(),
            };
        }

        let mut reduced = Vec::<Item>::with_capacity(tree.len());
        for item in tree {
            match item {
                Tree::Expression(expr) => reduced.push(Item::Expression(expr)),
                Tree::Punctuation(punct) => reduced.push(Item::Punctuation(punct)),
                Tree::Nested(nested, tree_span) => {
                    if let Some(ident) = reduced
                        .pop_if(|item| matches!(item, Item::Expression(Self::Variable(_, _))))
                    {
                        let Item::Expression(Self::Variable(ident, span)) = ident else {
                            unreachable!();
                        };
                        let expr =
                            Self::parse_call_expression(ident, span.join(tree_span), nested)?;
                        reduced.push(Item::Expression(expr));
                    } else {
                        let expr = Self::reduce_tree(nested)?;
                        reduced.push(Item::Expression(expr));
                    }
                }
            }
        }

        Self::reduce_ops(reduced)
    }

    fn parse_call_expression(
        callee: Arc<str>,
        span: Span,
        arguments: Vec<Tree>,
    ) -> Result<Self, ParseError> {
        if arguments.is_empty() {
            // 0 arguments
            return Ok(Self::Call(
                CallExpression {
                    callee,
                    arguments: Vec::new(),
                },
                span,
            ));
        }

        let mut commas: Vec<_> = arguments
            .iter()
            .enumerate()
            .filter_map(|(index, item)| match item {
                Tree::Punctuation(punct) if punct.as_str() == "," => Some((index, punct.span())),
                _ => None,
            })
            .collect();

        if commas.is_empty() {
            // 1 argument
            return Ok(Self::Call(
                CallExpression {
                    callee,
                    arguments: vec![Self::reduce_tree(arguments)?],
                },
                span,
            ));
        }

        // many arguments
        let first_comma_span = commas[0].1;
        let mut arguments_input = arguments;
        let mut arguments = VecDeque::<Self>::with_capacity(commas.len() + 1);
        while let Some((index, span)) = commas.pop() {
            let argument = arguments_input.split_off(index + 1);
            arguments_input.pop();
            if argument.is_empty() {
                return Err(ParseError::UnexpectedToken { span });
            }
            let argument = Self::reduce_tree(argument)?;
            arguments.push_front(argument);
        }

        if arguments_input.is_empty() {
            return Err(ParseError::UnexpectedToken {
                span: first_comma_span,
            });
        }
        arguments.push_front(Self::reduce_tree(arguments_input)?);

        Ok(Self::Call(
            CallExpression {
                callee,
                arguments: arguments.into(),
            },
            span,
        ))
    }

    fn reduce_ops(items: Vec<Item>) -> Result<Self, ParseError> {
        if items.is_empty() {
            return Err(ParseError::EndOfInput);
        }
        if items.len() == 1 {
            return match items.into_iter().next() {
                Some(Item::Expression(expr)) => Ok(expr),
                Some(Item::Punctuation(punct)) => {
                    Err(ParseError::UnexpectedToken { span: punct.span() })
                }
                None => unreachable!(),
            };
        }

        let items = Self::reduce_unary_ops(items)?;
        let items = Self::reduce_binary_ops(items)?;
        assert_eq!(items.len(), 1, "after reduce must be only one expression");

        let Some(Item::Expression(expr)) = items.into_iter().next() else {
            panic!("after reduce must be only one expression");
        };
        Ok(expr)
    }

    fn reduce_unary_ops(mut items: Vec<Item>) -> Result<Vec<Item>, ParseError> {
        const UNARY_OPS: &[&str] = &["!", "-"];

        let mut reduced = VecDeque::<Item>::with_capacity(items.len());
        while let Some(item) = items.pop() {
            match item {
                Item::Punctuation(punct) if UNARY_OPS.contains(&punct.as_str()) => {
                    match punct.as_str() {
                        "!" => {
                            let Some(Item::Expression(expr)) = reduced.pop_front() else {
                                return Err(ParseError::UnexpectedToken { span: punct.span() });
                            };
                            let span = punct.span().join(expr.span());
                            let expr = Box::new(expr);
                            reduced.push_front(Item::Expression(Self::Not(expr, span)));
                        }
                        "-" => {
                            // Binary minus (sub op) check
                            if let Some(Item::Expression(_)) = items.last() {
                                reduced.push_front(item);
                                continue;
                            }

                            let Some(Item::Expression(expr)) = reduced.pop_front() else {
                                return Err(ParseError::UnexpectedToken { span: punct.span() });
                            };
                            let span = punct.span().join(expr.span());
                            let expr = Box::new(expr);
                            reduced.push_front(Item::Expression(Self::Minus(expr, span)));
                        }
                        _ => unreachable!(),
                    }
                }
                item => {
                    reduced.push_front(item);
                }
            }
        }

        Ok(reduced.into())
    }

    fn reduce_binary_ops(mut items: Vec<Item>) -> Result<Vec<Item>, ParseError> {
        const BINARY_OPS_BY_PRIORITY: &[&[&str]] = &[
            &["*", "/", "%"],
            &["+", "-"],
            &["==", "!=", "<", ">", "<=", ">="],
            &["&&"],
            &["||"],
        ];

        for ops in BINARY_OPS_BY_PRIORITY {
            let mut reduced = Vec::<Item>::with_capacity(items.len());
            let mut iter = items.into_iter();
            while let Some(item) = iter.next() {
                match item {
                    Item::Punctuation(punct) if ops.contains(&punct.as_str()) => {
                        let Some(Item::Expression(expr_right)) = iter.next() else {
                            return Err(ParseError::UnexpectedToken { span: punct.span() });
                        };
                        let Some(Item::Expression(expr_left)) = reduced.pop() else {
                            return Err(ParseError::UnexpectedToken { span: punct.span() });
                        };
                        let span = punct.span().join(expr_left.span()).join(expr_right.span());
                        let expr_left = Box::new(expr_left);
                        let expr_right = Box::new(expr_right);
                        reduced.push(Item::Expression(match punct.as_str() {
                            "==" => Self::Eq(expr_left, expr_right, span),
                            "!=" => Self::Ne(expr_left, expr_right, span),
                            "<" => Self::Lt(expr_left, expr_right, span),
                            ">" => Self::Gt(expr_left, expr_right, span),
                            "<=" => Self::Le(expr_left, expr_right, span),
                            ">=" => Self::Ge(expr_left, expr_right, span),
                            "+" => Self::Add(expr_left, expr_right, span),
                            "-" => Self::Sub(expr_left, expr_right, span),
                            "*" => Self::Mul(expr_left, expr_right, span),
                            "/" => Self::Div(expr_left, expr_right, span),
                            "%" => Self::Mod(expr_left, expr_right, span),
                            "&&" => Self::And(expr_left, expr_right, span),
                            "||" => Self::Or(expr_left, expr_right, span),
                            _ => unreachable!(),
                        }));
                    }
                    item => {
                        reduced.push(item);
                    }
                }
            }

            items = reduced;
        }

        Ok(items)
    }
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Call(a, _), Self::Call(b, _)) => a == b,
            (Self::Variable(a, _), Self::Variable(b, _))
            | (Self::String(a, _), Self::String(b, _)) => a == b,
            (Self::Float(a, _), Self::Float(b, _)) => a == b,
            (Self::Int(a, _), Self::Int(b, _)) => a == b,
            (Self::Bool(a, _), Self::Bool(b, _)) => a == b,
            (Self::Not(a, _), Self::Not(b, _)) | (Self::Minus(a, _), Self::Minus(b, _)) => a == b,
            (Self::And(a, b, _), Self::And(c, d, _))
            | (Self::Or(a, b, _), Self::Or(c, d, _))
            | (Self::Eq(a, b, _), Self::Eq(c, d, _))
            | (Self::Ne(a, b, _), Self::Ne(c, d, _))
            | (Self::Gt(a, b, _), Self::Gt(c, d, _))
            | (Self::Ge(a, b, _), Self::Ge(c, d, _))
            | (Self::Lt(a, b, _), Self::Lt(c, d, _))
            | (Self::Le(a, b, _), Self::Le(c, d, _))
            | (Self::Add(a, b, _), Self::Add(c, d, _))
            | (Self::Sub(a, b, _), Self::Sub(c, d, _))
            | (Self::Mul(a, b, _), Self::Mul(c, d, _))
            | (Self::Div(a, b, _), Self::Div(c, d, _))
            | (Self::Mod(a, b, _), Self::Mod(c, d, _)) => a == c && b == d,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn tokens(expr: &str) -> TokenStream {
        let tokens: TokenStream = expr.parse().unwrap();
        tokens
    }

    #[test_case("var", &Expression::Variable(Arc::from("var"), Span::default()); "simple variable")]
    #[test_case("\"string\"", &Expression::String(Arc::from("string"), Span::default()); "simple string")]
    #[test_case("1", &Expression::Int(1, Span::default()); "simple int")]
    #[test_case("1.0", &Expression::Float(1.0, Span::default()); "simple float")]
    #[test_case("true", &Expression::Bool(true, Span::default()); "simple bool")]
    #[test_case("!var", &Expression::Not(Box::new(Expression::Variable(Arc::from("var"), Span::default())), Span::default()); "unary not var")]
    #[test_case("-var", &Expression::Minus(Box::new(Expression::Variable(Arc::from("var"), Span::default())), Span::default()); "unary minus var")]
    #[test_case("10 * -var", &Expression::Mul(
        Box::new(Expression::Int(10, Span::default())),
        Box::new(Expression::Minus(Box::new(Expression::Variable(Arc::from("var"), Span::default())), Span::default())),
        Span::default())
    ; "unary minus correctness")]
    #[test_case("10 -var", &Expression::Sub(
        Box::new(Expression::Int(10, Span::default())),
        Box::new(Expression::Variable(Arc::from("var"), Span::default())),
        Span::default())
    ; "binary minus correctness")]
    #[test_case("-!-!1", &Expression::Minus(
        Box::new(Expression::Not(
            Box::new(Expression::Minus(
                Box::new(Expression::Not(
                    Box::new(Expression::Int(1, Span::default())),
                    Span::default())),
                Span::default())),
            Span::default())),
        Span::default())
    ; "multiple unary ops")]
    #[test_case("f(a, x + y, !(true))", &Expression::Call(
        CallExpression {
            callee: Arc::from("f"),
            arguments: vec![
                Expression::Variable(Arc::from("a"), Span::default()),
                Expression::Add(
                    Box::new(Expression::Variable(Arc::from("x"), Span::default())),
                    Box::new(Expression::Variable(Arc::from("y"), Span::default())),
                    Span::default()),
                Expression::Not(Box::new(Expression::Bool(true, Span::default())), Span::default()),
            ],
        },
        Span::default())
    ; "function call")]
    fn test_expression(expr: &str, expected: &Expression) {
        let tokens = tokens(expr);
        let expr = Expression::parse(tokens).unwrap();
        assert_eq!(expr, *expected);
    }

    #[test_case("", None; "empty expression")]
    #[test_case("()", None; "empty parens")]
    #[test_case("a + () + b", None; "empty parens with other tokens")]
    #[test_case("((x)", None; "invalid parens sequence")]
    #[test_case("a)", Some(Span::new(1, 1)); "right paren without left")]
    #[test_case("a+*b", Some(Span::new(2, 2)); "two ops in a row")]
    #[test_case("*", Some(Span::new(0, 0)); "punctuation without expression")]
    #[test_case("f(,1,2)", Some(Span::new(2, 2)); "function call skipped first argument")]
    #[test_case("f(0,,2)", Some(Span::new(3, 3)); "function call skipped second argument")]
    #[test_case("f(0,1,)", Some(Span::new(5, 5)); "function call skipped last argument")]
    #[test_case("x + !*", Some(Span::new(4, 4)); "punctuation after unary not")]
    #[test_case("x + -*", Some(Span::new(4, 4)); "punctuation after unary minus")]
    fn invalid_expression(expr: &str, expected_error_span: Option<Span>) {
        let tokens = tokens(expr);
        let err = Expression::parse(tokens).unwrap_err();
        let error_span = match err {
            ParseError::UnexpectedToken { span } => Some(span),
            ParseError::EndOfInput => None,
        };
        assert_eq!(error_span, expected_error_span);
    }
}
