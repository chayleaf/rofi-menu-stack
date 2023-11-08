use serde::{de::Visitor, ser::SerializeStruct};
use serde::{Deserialize, Serialize};

use crate::fallback_row::FallbackRow;
use crate::{Data, DELIM};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Markup {
    Pango,
}

impl Markup {
    const ITEMS: &[&'static str] = &["pango"];
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Selection {
    /// Keep previously selected item
    Keep,
    /// Set selection to item X
    Set(i64),
}

struct SelectionVisitor;
impl<'a> Visitor<'a> for SelectionVisitor {
    type Value = Selection;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("selection")
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Keep)
    }
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Set(v.into()))
    }
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Set(v.into()))
    }
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Set(v.into()))
    }
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Set(v.into()))
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Set(v.into()))
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Set(v.into()))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Set(v))
    }
}

impl<'a> Deserialize<'a> for Selection {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(SelectionVisitor)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ModeOptions {
    /// Prompt text
    pub prompt: Option<String>,
    /// Message text (for explanations etc)
    pub message: Option<String>,
    /// Markup format
    pub markup: Option<Markup>,
    /// Selection mode
    pub selection: Option<Selection>,
    /// Next ROFI_DATA
    pub data: Data,
    /// Whether to autoselect the only item if there's only one item
    pub autoselect: bool,
}

impl ModeOptions {
    pub fn merge(&mut self, other: &ModeOptions) {
        if let Some(prompt) = &other.prompt {
            self.prompt = Some(prompt.clone());
        }
        if let Some(message) = &other.message {
            self.message = Some(message.clone());
        }
        if let Some(markup) = other.markup {
            self.markup = Some(markup);
        }
        if let Some(selection) = other.selection {
            self.selection = Some(selection);
        }
        if let Some(fallback) = &other.data.fallback {
            self.data.fallback = Some(fallback.clone());
        }
        if other.autoselect {
            self.autoselect = true;
        }
    }
    const FIELDS: &[&'static str] = &[
        "prompt",
        "message",
        "markup",
        "allow-custom",
        "selection",
        "autoselect",
    ];
    pub fn to_rofi(&self) -> String {
        let mut ret = String::new();
        if let Some(prompt) = &self.prompt {
            ret.push_str("\0prompt\x1F");
            ret.push_str(prompt);
            ret.push(DELIM);
        }
        if let Some(message) = &self.message {
            ret.push_str("\0message\x1F");
            ret.push_str(message);
            ret.push(DELIM);
        }
        #[allow(clippy::single_match)]
        match self.markup {
            Some(Markup::Pango) => {
                ret.push_str("\0markup-rows\x1Ftrue");
                ret.push(DELIM);
            }
            None => {}
        }
        if self.data.fallback.is_none() {
            ret.push_str("\0no-custom\x1Ftrue");
            ret.push(DELIM);
        }
        match self.selection {
            None => {}
            Some(Selection::Keep) => {
                ret.push_str("\0keep-selection\x1Ftrue");
                ret.push(DELIM);
            }
            Some(Selection::Set(new_sel)) => {
                ret.push_str("\0keep-selection\x1Ftrue");
                ret.push(DELIM);
                ret.push_str("\0new-selection\x1F");
                ret.push_str(&new_sel.to_string());
                ret.push(DELIM);
            }
        }
        ret.push_str("\0data\x1F");
        ret.push_str(&json5::to_string(&self.data).expect("failed to serialize data"));
        ret.push(DELIM);
        ret
    }
}

struct ModeOptionsVisitor;
impl<'a> Visitor<'a> for ModeOptionsVisitor {
    type Value = ModeOptions;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an object")
    }
    fn visit_map<A: serde::de::MapAccess<'a>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut ret = Self::Value::default();
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "prompt" => ret.prompt = Some(map.next_value()?),
                "message" => ret.message = Some(map.next_value()?),
                "markup" => match map.next_value::<String>()?.as_str() {
                    "pango" => ret.markup = Some(Markup::Pango),
                    key => return Err(serde::de::Error::unknown_variant(key, Markup::ITEMS)),
                },
                "fallback" => ret.data.fallback = Some(map.next_value::<FallbackRow>()?.0),
                "select" | "selection" => ret.selection = Some(map.next_value()?),
                "autoselect" => ret.autoselect = map.next_value()?,
                key => return Err(serde::de::Error::unknown_field(key, Self::Value::FIELDS)),
            }
        }
        Ok(ret)
    }
}

impl<'a> Deserialize<'a> for ModeOptions {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(ModeOptionsVisitor)
    }
}

impl Serialize for ModeOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let b2i = |x: bool| {
            if x {
                1
            } else {
                0
            }
        };
        let len = b2i(self.prompt.is_some())
            + b2i(self.message.is_some())
            + b2i(self.markup.is_some())
            + b2i(self.selection.is_some())
            + b2i(self.data.fallback.is_some())
            + b2i(self.autoselect);
        let mut s = serializer.serialize_struct("ModeOption", len)?;
        if let Some(prompt) = &self.prompt {
            s.serialize_field("prompt", &prompt)?;
        }
        if let Some(message) = &self.message {
            s.serialize_field("message", &message)?;
        }
        match self.markup {
            None => {}
            Some(Markup::Pango) => s.serialize_field("markup", "pango")?,
        }
        match self.selection {
            None => {}
            Some(Selection::Keep) => s.serialize_field("selection", "keep")?,
            Some(Selection::Set(x)) => s.serialize_field("select", &x)?,
        }
        if let Some(fallback) = &self.data.fallback {
            s.serialize_field("fallback", &fallback)?;
        }
        if self.autoselect {
            s.serialize_field("autoselect", &true)?;
        }
        s.end()
    }
}
