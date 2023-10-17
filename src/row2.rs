use serde::{de::Visitor, Deserialize, Serialize};

use crate::stack_op2::{FallbackStackOp, FallbackStackOps};

#[derive(Serialize)]
pub struct FallbackRow {
    pub next_script: Option<FallbackStackOp>,
    pub exec: String,
    pub fork: bool,
    pub ops: FallbackStackOps,
}

impl Default for FallbackRow {
    fn default() -> Self {
        Self {
            next_script: None,
            exec: "".to_owned(),
            fork: false,
            ops: FallbackStackOps(vec![]),
        }
    }
}

impl FallbackRow {
    const FIELDS: &[&'static str] = &["push", "jump", "exec", "fork"];

    pub fn info(&self) -> String {
        let mut ret = String::new();
        ret.push_str(&self.ops.to_string());
        if let Some(op0) = &self.next_script {
            if !self.ops.0.is_empty() {
                ret.push('\x03');
            }
            ret.push_str(&op0.to_string());
            if !self.exec.is_empty() {
                ret.push(';');
                if self.fork {
                    ret.push(';');
                }
                ret.push_str(&self.exec);
            }
        }
        ret
    }
}

struct RowVisitor;
impl<'a> Visitor<'a> for RowVisitor {
    type Value = FallbackRow;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string or an object")
    }
    fn visit_map<A: serde::de::MapAccess<'a>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut ret = Self::Value::default();
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "push" => ret.ops = map.next_value()?,
                "jump" => ret.next_script = Some(map.next_value()?),
                "exec" => ret.exec = map.next_value()?,
                "fork" => ret.fork = map.next_value()?,
                key => return Err(serde::de::Error::unknown_field(key, Self::Value::FIELDS)),
            }
        }
        Ok(ret)
    }
}

impl<'a> Deserialize<'a> for FallbackRow {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(RowVisitor)
    }
}
