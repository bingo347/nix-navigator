use serde::{Deserialize, Deserializer};
use std::fmt;

#[repr(transparent)]
pub struct DeserializeWrapper<T: DeserializeWrappable>(pub T);

pub trait DeserializeWrappable: Sized {
    type SourceType: for<'de> Deserialize<'de>;

    fn from_source(source: Self::SourceType) -> Result<Self, impl fmt::Display>;
}

impl<'de, T: DeserializeWrappable> Deserialize<'de> for DeserializeWrapper<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let source = T::SourceType::deserialize(deserializer)?;
        let value = T::from_source(source).map_err(serde::de::Error::custom)?;
        Ok(DeserializeWrapper(value))
    }
}
