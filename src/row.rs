use serde::{de::Visitor, Deserialize, Serialize};

use crate::Info;

#[derive(Serialize)]
pub struct Row {
    pub text: String,
    pub icon: String,
    pub meta: String,
    pub selectable: bool,
    pub urgent: bool,
    pub active: bool,
    pub info: Info,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            text: String::new(),
            icon: String::new(),
            meta: String::new(),
            selectable: true,
            info: Info::default(),
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
        "pop",
        "jump",
        "goto",
        "exec",
        "fork",
        "urgent",
        "active",
    ];

    pub fn info(&self) -> String {
        json5::to_string(&self.info).expect("failed to serialize row info")
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
        ret.push_str("info\x1F");
        ret.push_str(&self.info());
        ret.push('\x1F');
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
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "text" => ret.text = map.next_value()?,
                "icon" => ret.icon = map.next_value()?,
                "meta" => ret.meta = map.next_value()?,
                "selectable" => ret.selectable = map.next_value()?,
                "push" => ret.info.push_val = map.next_value()?,
                "pop" => ret.info.pop_val = map.next_value()?,
                "jump" => ret.info.push_script = map.next_value()?,
                "goto" => {
                    ret.info.pop_script = Some(1);
                    ret.info.push_script = map.next_value()?;
                }
                "return" => ret.info.pop_script = map.next_value()?,
                "exec" => ret.info.exec = map.next_value()?,
                "fork" => ret.info.fork = map.next_value()?,
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
