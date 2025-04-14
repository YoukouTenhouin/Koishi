use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub(crate) struct BoolAsInt {
    value: bool,
}

impl From<bool> for BoolAsInt {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl From<&BoolAsInt> for bool {
    fn from(value: &BoolAsInt) -> Self {
        value.value
    }
}

impl From<u8> for BoolAsInt {
    fn from(value: u8) -> Self {
        match value {
            0 => Self { value: false },
            _ => Self { value: true },
        }
    }
}

impl From<&BoolAsInt> for u8 {
    fn from(value: &BoolAsInt) -> Self {
        let ret: bool = value.into();
        ret as u8
    }
}

impl Display for BoolAsInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl Serialize for BoolAsInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value: u8 = self.into();
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BoolAsInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ret: BoolAsInt = u8::deserialize(deserializer)?.into();
        Ok(ret)
    }
}
