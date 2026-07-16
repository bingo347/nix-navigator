use super::{Expression, Template};
use crate::deserialize_wrapper::DeserializeWrapper;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize)]
pub struct Locale {
    #[serde(default, rename = "$rules")]
    pub rules: HashMap<String, Rule>,
    #[serde(default, rename = "$sections")]
    pub sections: HashMap<String, HashMap<String, FullTemplate>>,
}

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
        then: String,
    },
    Default {
        r#else: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FullTemplate {
    Simple(DeserializeWrapper<Template>),
    RuleSwitched {
        #[serde(rename = "$switch")]
        switch: DeserializeWrapper<Expression>,
        #[serde(flatten)]
        cases: HashMap<String, DeserializeWrapper<Template>>,
    },
}
