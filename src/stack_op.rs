use std::str::FromStr;

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Eq, PartialEq, Serialize)]
pub enum StackOp {
    Push(String),
    Pop,
}

impl ToString for StackOp {
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
        }
        ret
    }
}

#[derive(Serialize)]
pub struct StackOps(pub Vec<StackOp>);

impl ToString for StackOps {
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

impl FromStr for StackOp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('\x01') => Ok(Self::Push(s[1..].to_owned())),
            Some('\x02') => Ok(Self::Pop),
            _ => Err(()),
        }
    }
}

impl FromStr for StackOps {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = vec![];
        for v in s.split('\x03') {
            ret.push(v.parse::<StackOp>()?);
        }
        Ok(Self(ret))
    }
}

struct StackOpVisitor;
impl<'a> Visitor<'a> for StackOpVisitor {
    type Value = StackOp;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string/null/int")
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StackOp::Pop)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StackOp::Pop)
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(StackOp::Push(v.to_owned()))
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(StackOp::Push(v))
    }
    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StackOp::Push(v.to_owned()))
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
}

impl<'a> Deserialize<'a> for StackOp {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(StackOpVisitor)
    }
}

struct StackOpsVisitor;
impl<'a> Visitor<'a> for StackOpsVisitor {
    type Value = StackOps;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string/null/list of strings/null")
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StackOps(vec![StackOp::Pop]))
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StackOps(vec![StackOp::Pop]))
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(StackOps(vec![StackOp::Push(v.to_owned())]))
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(StackOps(vec![StackOp::Push(v)]))
    }
    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StackOps(vec![StackOp::Push(v.to_owned())]))
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
        while let Some(elem) = seq.next_element::<StackOp>()? {
            ret.push(elem);
        }
        Ok(StackOps(ret))
    }
}

impl<'a> Deserialize<'a> for StackOps {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(StackOpsVisitor)
    }
}
