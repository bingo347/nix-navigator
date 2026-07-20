use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
};
use std::{
    borrow::Cow,
    cmp::{Ordering, PartialEq, PartialOrd},
    fmt,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub enum Value {
    String(Arc<str>),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

struct ValueVisitor;

impl Value {
    #[must_use]
    pub fn cast_number(&self) -> Number {
        match self {
            Self::Integer(value) => Number::Integer(*value),
            Self::Float(value) => Number::Float(*value),
            Self::Boolean(true) => Number::Integer(1),
            Self::Boolean(false) => Number::Integer(0),
            Self::String(value) => value.parse::<i64>().map_or_else(
                |_| {
                    value
                        .parse::<f64>()
                        .map_or(Number::Float(f64::NAN), Number::Float)
                },
                Number::Integer,
            ),
        }
    }

    #[must_use]
    pub fn cast_boolean(&self) -> bool {
        match self {
            Self::Boolean(value) => *value,
            Self::Integer(value) => *value != 0,
            Self::Float(value) => *value != 0.0 || !f64::is_nan(*value),
            Self::String(value) => !value.is_empty(),
        }
    }
}

macro_rules! impl_cast {
    ($($($t:ty),* => $variant:ident;)*) => {
        $($(
            impl From<$t> for Value {
                fn from(value: $t) -> Self {
                    Value::$variant(value.into())
                }
            }
        )*)*
    };
}

impl_cast! {
    String, &str, Arc<str>, Box<str>, Cow<'_, str> => String;
    i8, i16, i32, i64, u8, u16, u32 => Integer;
    f32, f64 => Float;
    bool => Boolean;
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        if value < i64::MAX.cast_unsigned() {
            Value::Integer(value.cast_signed())
        } else {
            #[expect(clippy::cast_precision_loss)]
            Value::Float(value as f64)
        }
    }
}

impl From<Number> for Value {
    fn from(value: Number) -> Self {
        match value {
            Number::Integer(value) => Value::Integer(value),
            Number::Float(value) => Value::Float(value),
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if matches!(self, Value::String(_)) || matches!(rhs, Value::String(_)) {
            return format!("{self}{rhs}").into();
        }
        let this = self.cast_number();
        let rhs = rhs.cast_number();
        match (this, rhs) {
            (Number::Integer(this), Number::Integer(other)) => Value::Integer(this + other),
            (Number::Float(this), Number::Float(other)) => Value::Float(this + other),
            (Number::Integer(this), Number::Float(other)) => {
                Value::Float(cast_precision_loss(this) + other)
            }
            (Number::Float(this), Number::Integer(other)) => {
                Value::Float(this + cast_precision_loss(other))
            }
        }
    }
}

macro_rules! impl_number_ops {
    ($trait:ident $method:ident) => {
        impl $trait for Value {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self {
                let this = self.cast_number();
                let rhs = rhs.cast_number();
                match (this, rhs) {
                    (Number::Integer(this), Number::Integer(other)) => {
                        Value::Integer(this.$method(other))
                    }
                    (Number::Float(this), Number::Float(other)) => {
                        Value::Float(this.$method(other))
                    }
                    (Number::Integer(this), Number::Float(other)) => {
                        Value::Float(cast_precision_loss(this).$method(other))
                    }
                    (Number::Float(this), Number::Integer(other)) => {
                        Value::Float(this.$method(cast_precision_loss(other)))
                    }
                }
            }
        }
    };
}

impl_number_ops!(Sub sub);
impl_number_ops!(Mul mul);
impl_number_ops!(Div div);
impl_number_ops!(Rem rem);

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self {
        match self.cast_number() {
            Number::Integer(this) => Value::Integer(-this),
            Number::Float(this) => Value::Float(-this),
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Self {
        Self::Boolean(!self.cast_boolean())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::String(this), Self::String(other)) => Some(this.cmp(other)),
            (Self::String(this), other) => Some(this.as_ref().cmp(other.to_string().as_str())),
            (this, Self::String(other)) => Some(this.to_string().as_str().cmp(other.as_ref())),
            (this, other) => {
                let this = this.cast_number();
                let other = other.cast_number();
                match (this, other) {
                    (Number::Integer(this), Number::Integer(other)) => Some(this.cmp(&other)),
                    (Number::Float(this), Number::Float(other)) => this.partial_cmp(&other),
                    (Number::Float(this), Number::Integer(other)) => {
                        this.partial_cmp(&cast_precision_loss(other))
                    }
                    (Number::Integer(this), Number::Float(other)) => {
                        cast_precision_loss(this).partial_cmp(&other)
                    }
                }
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(value) => f.write_str(value),
            Value::Integer(value) => fmt::Display::fmt(value, f),
            Value::Float(value) => fmt::Display::fmt(value, f),
            Value::Boolean(value) => fmt::Display::fmt(value, f),
        }
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::String(value) => serializer.serialize_str(value),
            Value::Integer(value) => serializer.serialize_i64(*value),
            Value::Float(value) => serializer.serialize_f64(*value),
            Value::Boolean(value) => serializer.serialize_bool(*value),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Visitor<'_> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("valid expression value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Boolean(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Integer(v))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        Ok(Value::Float(v))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(Arc::from(v)))
    }
}

const fn cast_precision_loss(v: i64) -> f64 {
    #[expect(clippy::cast_precision_loss)]
    {
        v as f64
    }
}
