use std::{borrow::Cow, collections::HashMap, sync::Arc};

mod expression;
mod value;

pub use crate::{
    expression::{Expression, UnaryOperation},
    value::Value,
};

pub type VariablesMap = HashMap<Cow<'static, str>, Value>;

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("Variable `{0}` not found")]
    VariableNotFound(Arc<str>),
}
