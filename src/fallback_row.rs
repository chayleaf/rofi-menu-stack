use serde::{de::Visitor, Deserialize, Serialize};

use crate::Info;

#[derive(Default, Serialize)]
pub struct FallbackRow(Info);

impl FallbackRow {
    const FIELDS: &[&'static str] = &["push", "pop", "jump", "goto", "return", "exec", "fork"];
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
                "push" => ret.0.push_val = map.next_value()?,
                "pop" => ret.0.pop_val = map.next_value()?,
                "jump" => ret.0.push_script = map.next_value()?,
                "goto" => {
                    if let Some(x) = &mut ret.0.pop_script {
                        *x += 1;
                    }
                    ret.0.push_script = map.next_value()?;
                }
                "return" => ret.0.pop_script = map.next_value()?,
                "exec" => ret.0.exec = map.next_value()?,
                "fork" => ret.0.fork = map.next_value()?,
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
