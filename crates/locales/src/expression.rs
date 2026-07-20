use crate::Value;
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

#[derive(Debug, Clone, Copy)]
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
