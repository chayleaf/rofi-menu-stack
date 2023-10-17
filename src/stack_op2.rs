use std::str::FromStr;

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Eq, PartialEq, Serialize)]
pub enum FallbackStackOp {
    Push(String),
    PushUser,
    Pop,
}

impl FallbackStackOp {
    pub fn apply(self, user: &str, stack: &mut Vec<String>) {
        match self {
            Self::Push(x) => stack.push(x),
            Self::PushUser => stack.push(user.to_owned()),
            Self::Pop => {
                stack.pop();
            }
        }
    }
}

impl ToString for FallbackStackOp {
    fn to_string(&self) -> String {
        let mut ret = "".to_owned();
        match self {
            Self::Push(x) => {
                ret.push('\x01');
                ret.push_str(x);
            }
            Self::Pop => {
                ret.push('\x02');
            }
            Self::PushUser => {
                ret.push('\x05');
            }
        }
        ret
    }
}

#[derive(Serialize)]
pub struct FallbackStackOps(pub Vec<FallbackStackOp>);

impl ToString for FallbackStackOps {
    fn to_string(&self) -> String {
        let mut ret = "".to_owned();
        for op in &self.0 {
            ret += &op.to_string();
            ret.push('\x03');
        }
        ret.pop();
        ret
    }
}

impl FromStr for FallbackStackOp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('\x01') => Ok(Self::Push(s[1..].to_owned())),
            Some('\x02') => Ok(Self::Pop),
            Some('\x05') => Ok(Self::PushUser),
            _ => Err(()),
        }
    }
}

impl FromStr for FallbackStackOps {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = vec![];
        for v in s.split('\x03') {
            ret.push(v.parse::<FallbackStackOp>()?);
        }
        Ok(Self(ret))
    }
}

struct StackOpVisitor;
impl<'a> Visitor<'a> for StackOpVisitor {
    type Value = FallbackStackOp;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string/null/int")
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::Pop)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::Pop)
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(FallbackStackOp::Push(v.to_owned()))
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(FallbackStackOp::Push(v))
    }
    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::Push(v.to_owned()))
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_any(self)
    }
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_any(self)
    }
    fn visit_i8<E>(self, _: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
    fn visit_u8<E>(self, _: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
    fn visit_i16<E>(self, _: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
    fn visit_u16<E>(self, _: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
    fn visit_i32<E>(self, _: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
    fn visit_u32<E>(self, _: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
    fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
    fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOp::PushUser)
    }
}

impl<'a> Deserialize<'a> for FallbackStackOp {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(StackOpVisitor)
    }
}

struct StackOpsVisitor;
impl<'a> Visitor<'a> for StackOpsVisitor {
    type Value = FallbackStackOps;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string/null/list of strings/null")
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::Pop]))
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::Pop]))
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(FallbackStackOps(vec![FallbackStackOp::Push(v.to_owned())]))
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(FallbackStackOps(vec![FallbackStackOp::Push(v)]))
    }
    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::Push(v.to_owned())]))
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_any(self)
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut ret = vec![];
        while let Some(elem) = seq.next_element::<FallbackStackOp>()? {
            ret.push(elem);
        }
        Ok(FallbackStackOps(ret))
    }
    fn visit_i8<E>(self, _: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
    fn visit_u8<E>(self, _: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
    fn visit_i16<E>(self, _: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
    fn visit_u16<E>(self, _: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
    fn visit_i32<E>(self, _: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
    fn visit_u32<E>(self, _: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
    fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
    fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FallbackStackOps(vec![FallbackStackOp::PushUser]))
    }
}

impl<'a> Deserialize<'a> for FallbackStackOps {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(StackOpsVisitor)
    }
}
