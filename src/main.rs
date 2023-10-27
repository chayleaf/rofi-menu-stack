use fork::Fork;
use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};
use std::{
    env,
    io::{stdout, BufRead, BufReader, Write},
    process::{Command, Stdio},
};

mod fallback_row;
mod row;

use fallback_row::FallbackRow;
use row::Row;

const DELIM: char = '\x0b';

#[derive(Debug, Default, Eq, PartialEq)]
enum Selection {
    /// Reset selection to the first item
    #[default]
    Reset,
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

#[derive(Debug, Default, Eq, PartialEq)]
enum Markup {
    #[default]
    None,
    Pango,
}

impl Markup {
    const FIELDS: &[&'static str] = &["pango"];
}

struct ModeOptions {
    /// Prompt text
    prompt: String,
    /// Message text (for explanations etc)
    message: String,
    /// Markup format
    markup: Markup,
    /// Selection mode
    selection: Selection,
    /// Next ROFI_DATA
    data: Data,
}

impl Default for ModeOptions {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            message: String::new(),
            markup: Markup::None,
            selection: Selection::Reset,
            data: Data::default(),
        }
    }
}

impl ModeOptions {
    const FIELDS: &[&'static str] = &["prompt", "message", "markup", "allow-custom", "selection"];
    fn to_rofi(&self) -> String {
        let mut ret = String::new();
        if !self.prompt.is_empty() {
            ret.push_str("\0prompt\x1F");
            ret.push_str(&self.prompt);
            ret.push(DELIM);
        }
        if !self.message.is_empty() {
            ret.push_str("\0message\x1F");
            ret.push_str(&self.message);
            ret.push(DELIM);
        }
        if self.markup == Markup::Pango {
            ret.push_str("\0markup-rows\x1Ftrue");
            ret.push(DELIM);
        }
        if self.data.fallback.is_none() {
            ret.push_str("\0no-custom\x1Ftrue");
            ret.push(DELIM);
        }
        match self.selection {
            Selection::Reset => {}
            Selection::Keep => {
                ret.push_str("\0keep-selection\x1Ftrue");
                ret.push(DELIM);
            }
            Selection::Set(new_sel) => {
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
                "prompt" => ret.prompt = map.next_value()?,
                "message" => ret.message = map.next_value()?,
                "markup" => match map.next_value::<String>()?.as_str() {
                    "pango" => ret.markup = Markup::Pango,
                    key => return Err(serde::de::Error::unknown_variant(key, Markup::FIELDS)),
                },
                "fallback" => ret.data.fallback = Some(map.next_value::<FallbackRow>()?.0),
                "select" | "selection" => ret.selection = map.next_value()?,
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

#[derive(Clone, Debug)]
pub enum VecString {
    Multi(Vec<VecString>),
    Single(String),
    UserInput,
}

impl VecString {
    fn is_empty(&self) -> bool {
        matches!(self, Self::Multi(x) if x.is_empty())
    }
    fn flatten(&self, input: &str) -> String {
        match self {
            Self::Multi(v) => v.iter().map(|x| x.flatten(input)).collect::<String>(),
            Self::Single(s) => s.clone(),
            Self::UserInput => input.to_owned(),
        }
    }
    fn flatten1(&self, input: &str) -> Vec<String> {
        match self {
            Self::Multi(v) => v.iter().map(|x| x.flatten(input)).collect(),
            Self::Single(s) => vec![s.clone()],
            Self::UserInput => vec![input.to_owned()],
        }
    }
}

impl Serialize for VecString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::UserInput => serializer.serialize_none(),
            Self::Single(s) => serializer.serialize_some(s),
            Self::Multi(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for x in v {
                    seq.serialize_element(x)?;
                }
                seq.end()
            }
        }
    }
}

struct VecStringVisitor;
impl<'a> Visitor<'a> for VecStringVisitor {
    type Value = VecString;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a list of strings")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(VecString::Single(v.to_owned()))
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(VecString::Single(v))
    }
    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(VecString::Single(v.to_owned()))
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(VecString::UserInput)
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(VecString::UserInput)
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut ret = vec![];
        while let Some(val) = seq.next_element()? {
            ret.push(val);
        }
        Ok(VecString::Multi(ret))
    }
}
impl<'de> Deserialize<'de> for VecString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(VecStringVisitor)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Data {
    stack: Vec<String>,
    call_stack: Vec<String>,
    fallback: Option<Info>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Info {
    pub push_call: VecString,
    pub push: VecString,
    pub pop_call: Option<usize>,
    pub pop: Option<usize>,
    pub exec: VecString,
    pub fork: bool,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            push: VecString::Multi(vec![]),
            push_call: VecString::Multi(vec![]),
            pop: Some(0),
            pop_call: Some(0),
            exec: VecString::Multi(vec![]),
            fork: false,
        }
    }
}

fn parse_var(var: String) -> Result<Vec<String>, json5::Error> {
    let v = var.trim();
    if v.starts_with('[') && v.ends_with(']') {
        json5::from_str(&var)
    } else {
        Ok(vec![var])
    }
}

fn main() {
    // 0: init
    // 1: selected entry
    // 2: selected custom entry
    // 10-28: custom keybindings 1-19
    // let retv = env::var("ROFI_RETV").ok();
    // common info
    let data = env::var("ROFI_DATA").ok();
    // row info
    let info = env::var("ROFI_INFO").ok();
    // row text
    let input = env::args().nth(1);
    let first_launch = info.is_none() && data.is_none();
    if first_launch {
        if let Some(input) = input {
            #[allow(clippy::single_match)]
            match input.as_str() {
                "unjson5" => {
                    println!(
                        "{}",
                        serde_json::to_string(
                            &json5::from_str::<serde_json::Value>(
                                &env::args()
                                    .nth(2)
                                    .expect("provide json5 to convert to json")
                            )
                            .expect("invalid json5")
                        )
                        .expect("failed to serialize json")
                    );
                }
                _ => {}
            }
            return;
        }
    }
    let enable_debug = cfg!(debug_assertions);
    if enable_debug {
        eprintln!("data {data:?}, info {info:?}");
    }
    let mut data: Data = json5::from_str(&data.unwrap_or_default()).unwrap_or_default();
    let info: Info = info.as_deref().map_or_else(
        || data.fallback.clone().unwrap_or_default(),
        |info| json5::from_str(info).expect("failed to parse info"),
    );
    let input = input.as_deref().unwrap_or_default();
    if !info.exec.is_empty()
        && (!info.fork
            || (matches!(fork::daemon(true, true), Ok(Fork::Child)) && {
                let _ = fork::close_fd();
                true
            }))
    {
        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        if matches!(info.exec, VecString::Multi(_)) {
            cmd.arg("\"$0\" \"$@\"").args(info.exec.flatten1(input));
        } else {
            cmd.arg(info.exec.flatten(input));
        }
        if let Ok(mut proc) = cmd.spawn() {
            let _ = proc.wait();
        }
        if info.fork {
            return;
        }
    }
    if data.call_stack.is_empty() {
        if enable_debug {
            eprintln!("pushing initial_script");
        }
        data.call_stack.extend(
            parse_var(
                env::var("INITIAL_SCRIPT")
                    .expect("INITIAL_SCRIPT must be set as the default submenu to call"),
            )
            .expect("INITIAL_SCRIPT must be valid json5"),
        );
        if let Ok(x) = env::var("INITIAL_STACK") {
            data.stack = parse_var(x).expect("INITIAL_STACK must be valid json5");
        }
    }
    if let Some(x) = info.pop {
        if x <= data.stack.len() {
            data.stack.truncate(data.stack.len() - x);
        } else {
            return;
        }
    } else {
        data.stack.clear();
    }
    for x in info.push.flatten1(input) {
        data.stack.push(x);
    }
    if let Some(x) = info.pop_call {
        if x <= data.call_stack.len() {
            data.call_stack.truncate(data.call_stack.len() - x);
        } else {
            return;
        }
    } else {
        data.call_stack.clear();
    }
    for x in info.push_call.flatten1(input) {
        data.call_stack.push(x);
    }
    if enable_debug {
        eprintln!("data {data:?}, info {info:?}");
    }
    let Some(argv0) = data.call_stack.last() else {
        return;
    };
    let mut cmd = Command::new("bash");
    cmd.arg("-c").arg("\"$0\" \"$@\"").arg(argv0).env("_CALL_STACK_LEN", (data.call_stack.len() - 1).to_string());
    if enable_debug {
        data.stack.reverse();
        eprintln!("passing args {:?}", data.stack);
        data.stack.reverse();
    }
    cmd.args(data.stack.iter().rev());
    cmd.stdout(Stdio::piped());
    let cmd = cmd.spawn().expect("failed to spawn script");
    let mut buf = BufReader::new(cmd.stdout.expect("script is missing stdout?"));
    let mut line = String::new();
    let mut out = stdout().lock();
    buf.read_line(&mut line)
        .expect("failed to read menu options");
    if first_launch {
        out.write_all(b"\0delim\x1F")
            .expect("failed writing into stdout");
        out.write_all(&[DELIM as u8])
            .expect("failed writing into stdout");
        out.write_all(b"\n").expect("failed writing into stdout");
    }
    if enable_debug {
        eprintln!("opts: {line:?}");
    }
    let mut opts: ModeOptions = json5::from_str(&line).expect("failed to parse menu options");
    opts.data.call_stack = data.call_stack;
    opts.data.stack = data.stack;
    out.write_all(opts.to_rofi().as_bytes())
        .expect("failed writing menu options into stdout");
    let mut first = true;
    while let Ok(len) = buf.read_line({
        line.clear();
        &mut line
    }) {
        let line = &line[..len];
        if line.is_empty() {
            break;
        }
        if enable_debug {
            eprintln!("got a row {line:?}");
        }
        match json5::from_str::<Row>(line) {
            Ok(row) => {
                if let Some(row) = row.to_rofi() {
                    if first {
                        first = false;
                    } else {
                        out.write_all(&[DELIM as u8])
                            .expect("failed writing into stdout");
                    }
                    out.write_all(row.as_bytes())
                        .expect("failed writing into stdout");
                }
            }
            Err(err) => {
                eprintln!("row parse error ({line}):\n{err}");
            }
        }
    }
}
