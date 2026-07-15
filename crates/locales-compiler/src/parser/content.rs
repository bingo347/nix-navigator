use super::Expression;
use crate::deserialize_wrapper::DeserializeWrapper;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub args: HashMap<String, RuleArgumentType>,
    pub r#do: Vec<RuleVariant>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleArgumentType {
    Str,
    Float,
    Int,
    Bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RuleVariant {
    Conditional {
        r#if: DeserializeWrapper<Expression>,
        r#then: String,
    },
    Default {
        r#else: String,
    },
}
