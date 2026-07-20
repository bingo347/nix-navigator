use crate::{EvalError, Value, VariablesMap};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "k", content = "e")]
pub enum Expression {
    #[serde(rename = "f()")]
    Call {
        #[serde(rename = "c")]
        callee: Arc<str>,
        #[serde(rename = "a")]
        args: Vec<Expression>,
    },
    #[serde(rename = "var")]
    Variable(Arc<str>),
    #[serde(rename = "val")]
    Value(Value),
    #[serde(rename = "1op")]
    UnaryOperation(UnaryOperation, Box<Expression>),
    #[serde(rename = "2op")]
    BinaryOperation(Box<Expression>, BinaryOperation, Box<Expression>),
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum UnaryOperation {
    Not = b'!',
    Minus = b'-',
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum BinaryOperation {
    Add = u16::from_be_bytes(*b"\0+"),
    Sub = u16::from_be_bytes(*b"\0-"),
    Mul = u16::from_be_bytes(*b"\0*"),
    Div = u16::from_be_bytes(*b"\0/"),
    Rem = u16::from_be_bytes(*b"\0%"),
    And = u16::from_be_bytes(*b"&&"),
    Or = u16::from_be_bytes(*b"||"),
    Eq = u16::from_be_bytes(*b"=="),
    Ne = u16::from_be_bytes(*b"!="),
    Gt = u16::from_be_bytes(*b"\0>"),
    Ge = u16::from_be_bytes(*b">="),
    Lt = u16::from_be_bytes(*b"\0<"),
    Le = u16::from_be_bytes(*b"<="),
}

impl Expression {
    pub fn eval(
        &self,
        variables: &VariablesMap,
        mut call_handler: impl FnMut(&str, Vec<Value>) -> Result<Value, EvalError>,
    ) -> Result<Value, EvalError> {
        self.eval_inner(variables, &mut call_handler)
    }

    fn eval_inner(
        &self,
        variables: &VariablesMap,
        call_handler: &mut impl FnMut(&str, Vec<Value>) -> Result<Value, EvalError>,
    ) -> Result<Value, EvalError> {
        match self {
            Self::Call { callee, args } => {
                let args = args
                    .iter()
                    .map(|arg| arg.eval_inner(variables, call_handler))
                    .collect::<Result<_, _>>()?;
                call_handler(callee, args)
            }
            Self::Variable(var) => variables
                .get(var.as_ref())
                .cloned()
                .ok_or_else(|| EvalError::VariableNotFound(Arc::clone(var))),
            Self::Value(value) => Ok(value.clone()),
            Self::UnaryOperation(op, expr) => {
                let value = expr.eval_inner(variables, call_handler)?;
                Ok(match op {
                    UnaryOperation::Not => !value,
                    UnaryOperation::Minus => -value,
                })
            }
            Self::BinaryOperation(left, op, right) => {
                let left = left.eval_inner(variables, call_handler)?;
                if *op == BinaryOperation::And && !left.cast_boolean() {
                    return Ok(left);
                }
                if *op == BinaryOperation::Or && left.cast_boolean() {
                    return Ok(left);
                }
                let right = right.eval_inner(variables, call_handler)?;
                Ok(match op {
                    BinaryOperation::And | BinaryOperation::Or => right,
                    BinaryOperation::Add => left + right,
                    BinaryOperation::Sub => left - right,
                    BinaryOperation::Mul => left * right,
                    BinaryOperation::Div => left / right,
                    BinaryOperation::Rem => left % right,
                    BinaryOperation::Eq => (left == right).into(),
                    BinaryOperation::Ne => (left != right).into(),
                    BinaryOperation::Gt => (left > right).into(),
                    BinaryOperation::Ge => (left >= right).into(),
                    BinaryOperation::Lt => (left < right).into(),
                    BinaryOperation::Le => (left <= right).into(),
                })
            }
        }
    }
}

impl Serialize for UnaryOperation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self as u8)
    }
}

impl Serialize for BinaryOperation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(*self as u16)
    }
}

impl<'de> Deserialize<'de> for UnaryOperation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        Ok(match value {
            b'!' => UnaryOperation::Not,
            b'-' => UnaryOperation::Minus,
            value => {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Unsigned(value.into()),
                    &format!("{} or {}", b'!', b'-').as_str(),
                ));
            }
        })
    }
}

impl<'de> Deserialize<'de> for BinaryOperation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        const VALUES: [BinaryOperation; 13] = [
            BinaryOperation::Add,
            BinaryOperation::Sub,
            BinaryOperation::Mul,
            BinaryOperation::Div,
            BinaryOperation::Rem,
            BinaryOperation::And,
            BinaryOperation::Or,
            BinaryOperation::Eq,
            BinaryOperation::Ne,
            BinaryOperation::Gt,
            BinaryOperation::Ge,
            BinaryOperation::Lt,
            BinaryOperation::Le,
        ];

        let value = u16::deserialize(deserializer)?;

        for op in VALUES {
            if value == op as u16 {
                return Ok(op);
            }
        }

        Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(value.into()),
            &format!(
                "one of {}",
                std::fmt::from_fn(|f| {
                    let mut first = true;
                    for op in VALUES {
                        if !first {
                            f.write_str(", ")?;
                        }
                        first = false;
                        f.write_fmt(format_args!("{}", op as u16))?;
                    }
                    Ok(())
                })
            )
            .as_str(),
        ))
    }
}

#[test]
fn eval_expression() {
    let expr = Expression::BinaryOperation(
        Box::new(Expression::Value("Hello ".into())),
        BinaryOperation::Add,
        Box::new(Expression::BinaryOperation(
            Box::new(Expression::Variable("a".into())),
            BinaryOperation::Mul,
            Box::new(Expression::UnaryOperation(
                UnaryOperation::Minus,
                Box::new(Expression::Call {
                    callee: "f".into(),
                    args: vec![
                        Expression::Variable("b".into()),
                        Expression::BinaryOperation(
                            Box::new(Expression::Value(0.into())),
                            BinaryOperation::And,
                            Box::new(Expression::Call {
                                callee: "never".into(),
                                args: vec![],
                            }),
                        ),
                        Expression::BinaryOperation(
                            Box::new(Expression::Value(1.into())),
                            BinaryOperation::Or,
                            Box::new(Expression::Call {
                                callee: "never".into(),
                                args: vec![],
                            }),
                        ),
                    ],
                }),
            )),
        )),
    );

    let variables = VariablesMap::from_iter([("a".into(), 2.into()), ("b".into(), 3.into())]);

    assert_eq!(
        expr.eval(&variables, |name, args| {
            assert_eq!(name, "f");
            assert_eq!(args, [3.into(), 0.into(), 1.into()]);
            Ok(6.into())
        })
        .unwrap(),
        "Hello -12".into()
    );
}
