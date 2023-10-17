use serde::{de::Visitor, Deserialize, Serialize};

use crate::stack_op::{StackOp, StackOps};

#[derive(Serialize)]
pub struct Row {
    pub text: String,
    pub icon: String,
    pub meta: String,
    pub selectable: bool,
    pub next_script: Option<StackOp>,
    pub exec: String,
    pub fork: bool,
    pub ops: StackOps,
    pub urgent: bool,
    pub active: bool,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            text: "".to_owned(),
            icon: "".to_owned(),
            meta: "".to_owned(),
            selectable: true,
            next_script: None,
            exec: "".to_owned(),
            fork: false,
            ops: StackOps(vec![]),
            urgent: false,
            active: false,
        }
    }
}

impl Row {
    const FIELDS: &[&'static str] = &[
        "text",
        "icon",
        "meta",
        "selectable",
        "push",
        "jump",
        "exec",
        "fork",
        "urgent",
        "active",
    ];

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

    pub fn to_rofi(&self) -> Option<String> {
        if self.text.is_empty() {
            return None;
        }
        let mut ret = self.text.clone();
        ret.push('\0');
        if !self.icon.is_empty() {
            ret.push_str("icon\x1F");
            ret.push_str(&self.icon);
            ret.push('\x1F');
        }
        if !self.meta.is_empty() {
            ret.push_str("meta\x1F");
            ret.push_str(&self.meta);
            ret.push('\x1F');
        }
        if !self.selectable {
            ret.push_str("nonselectable\x1Ftrue\x1F");
        }
        if !self.ops.0.is_empty() || self.next_script.is_some() {
            ret.push_str("info\x1F");
            ret.push_str(&self.info());
            ret.push('\x1F');
        }
        if self.urgent {
            ret.push_str("urgent\x1Ftrue\x1F");
        }
        if self.active {
            ret.push_str("active\x1Ftrue\x1F");
        }
        ret.pop();
        Some(ret)
    }
}

struct RowVisitor;
impl<'a> Visitor<'a> for RowVisitor {
    type Value = Row;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string or an object")
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Self::Value {
            text: v.to_owned(),
            ..Self::Value::default()
        })
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(Self::Value {
            text: v,
            ..Self::Value::default()
        })
    }
    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value {
            text: v.to_owned(),
            ..Self::Value::default()
        })
    }
    fn visit_map<A: serde::de::MapAccess<'a>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut ret = Self::Value::default();
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "text" => ret.text = map.next_value()?,
                "icon" => ret.icon = map.next_value()?,
                "meta" => ret.meta = map.next_value()?,
                "selectable" => ret.selectable = map.next_value()?,
                "push" => ret.ops = map.next_value()?,
                "jump" => ret.next_script = Some(map.next_value()?),
                "exec" => ret.exec = map.next_value()?,
                "fork" => ret.fork = map.next_value()?,
                "urgent" => ret.urgent = map.next_value()?,
                "active" => ret.active = map.next_value()?,
                key => return Err(serde::de::Error::unknown_field(key, Self::Value::FIELDS)),
            }
        }
        Ok(ret)
    }
}

impl<'a> Deserialize<'a> for Row {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(RowVisitor)
    }
}
