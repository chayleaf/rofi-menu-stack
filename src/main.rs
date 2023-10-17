use fork::{daemon, Fork};
use serde::{de::Visitor, Deserialize};
use std::{
    env,
    io::{stdout, BufRead, BufReader, Write, Read, stderr, stdin},
    process::{Command, Stdio},
    str::FromStr, os::fd::AsRawFd,
};

const DELIM: &str = "\x0b";

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
    fn visit_unit<E>(self) -> Result<Self::Value, E> where E: serde::de::Error {
        Ok(Selection::Keep)
    }
    /*fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> where E: serde::de::Error {
        Ok(Selection::Set(v as i64))
    }
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: serde::de::Error {
        Ok(Selection::Set(v as i64))
    }*/
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v.into()))
    }
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v.into()))
    }
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v.into()))
    }
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v.into()))
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v.into()))
    }
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v.into()))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v as i64))
    }
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v as i64))
    }
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(Selection::Set(v as i64))
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
    /// Whether to allow freeform text input
    allow_custom: bool,
    /// Selection mode
    selection: Selection,
    /// Next ROFI_DATA
    data: String,
}

impl Default for ModeOptions {
    fn default() -> Self {
        Self {
            prompt: "".to_owned(),
            message: "".to_owned(),
            markup: Markup::None,
            allow_custom: false,
            selection: Selection::Reset,
            data: "".to_owned(),
        }
    }
}

impl ModeOptions {
    const FIELDS: &[&'static str] = &["prompt", "message", "markup", "allow-custom", "selection"];
    fn to_rofi(&self) -> String {
        let mut ret = "".to_owned();
        if !self.prompt.is_empty() {
            ret.push_str("\0prompt\x1F");
            ret.push_str(&self.prompt);
            ret.push_str(DELIM);
        }
        if !self.message.is_empty() {
            ret.push_str("\0message\x1F");
            ret.push_str(&self.message);
            ret.push_str(DELIM);
        }
        if self.markup == Markup::Pango {
            ret.push_str("\0markup-rows\x1Ftrue");
            ret.push_str(DELIM);
        }
        if !self.allow_custom {
            ret.push_str("\0no-custom\x1Ftrue");
            ret.push_str(DELIM);
        }
        match self.selection {
            Selection::Reset => {}
            Selection::Keep => {
                ret.push_str("\0keep-selection\x1Ftrue");
                ret.push_str(DELIM);
            }
            Selection::Set(new_sel) => {
                ret.push_str("\0keep-selection\x1Ftrue");
                ret.push_str(DELIM);
                ret.push_str("\0new-selection\x1F");
                ret.push_str(&new_sel.to_string());
                ret.push_str(DELIM);
            }
        }
        if !self.data.is_empty() {
            ret.push_str("\0data\x1F");
            ret.push_str(&self.data);
            ret.push_str(DELIM);
        }
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
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "prompt" => ret.prompt = map.next_value()?,
                "message" => ret.message = map.next_value()?,
                "markup" => match map.next_value::<&str>()? {
                    "pango" => ret.markup = Markup::Pango,
                    key => return Err(serde::de::Error::unknown_variant(key, Markup::FIELDS)),
                },
                "allow-custom" => ret.allow_custom = map.next_value()?,
                "selection" => ret.selection = map.next_value()?,
                // TODO: selection
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

#[derive(Eq, PartialEq)]
enum StackOp {
    Push(Option<String>),
    Pop,
}

impl StackOp {
    fn apply(self, user_input: &str, stack: &mut Vec<String>) {
        match self {
            Self::Push(Some(x)) => stack.push(x),
            Self::Push(None) => stack.push(user_input.to_owned()),
            Self::Pop => {
                stack.pop();
            }
        }
    }
}

impl ToString for StackOp {
    fn to_string(&self) -> String {
        let mut ret = "".to_owned();
        match self {
            Self::Push(Some(x)) => {
                ret.push('\x01');
                ret.push_str(x);
            }
            Self::Pop => {
                ret.push('\x02');
            }
            Self::Push(None) => {
                ret.push('\x04');
            }
        }
        ret
    }
}

struct StackOps(Vec<StackOp>);

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
            Some('\x01') => Ok(Self::Push(Some(s[1..].to_owned()))),
            Some('\x02') => Ok(Self::Pop),
            Some('\x04') => Ok(Self::Push(None)),
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
    fn visit_unit<E>(self) -> Result<Self::Value, E> where E: serde::de::Error {
        Ok(StackOp::Pop)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StackOp::Pop)
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(StackOp::Push(Some(v.to_owned())))
    }
    fn visit_i8<E>(self, _: i8) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_u8<E>(self, _: u8) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_i16<E>(self, _: i16) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_u16<E>(self, _: u16) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_i32<E>(self, _: i32) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_u32<E>(self, _: u32) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOp::Push(None))
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(StackOp::Push(Some(v)))
    }
    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StackOp::Push(Some(v.to_owned())))
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_any(self)
    }
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: serde::Deserializer<'a> {
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
        Ok(StackOps(vec![StackOp::Push(Some(v.to_owned()))]))
    }
    fn visit_i8<E>(self, _: i8) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_u8<E>(self, _: u8) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_i16<E>(self, _: i16) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_u16<E>(self, _: u16) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_i32<E>(self, _: i32) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_u32<E>(self, _: u32) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E> where E: serde::de::Error, {
        Ok(StackOps(vec![StackOp::Push(None)]))
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(StackOps(vec![StackOp::Push(Some(v))]))
    }
    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StackOps(vec![StackOp::Push(Some(v.to_owned()))]))
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

struct Row {
    text: String,
    icon: String,
    meta: String,
    selectable: bool,
    next_script: Option<StackOp>,
    exec: String,
    fork: bool,
    ops: StackOps,
    urgent: bool,
    active: bool,
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

    fn to_rofi(&self) -> Option<String> {
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
                "push" => {
                    ret.ops = map.next_value()?;
                }
                "jump" => {
                    ret.next_script = Some(map.next_value::<StackOp>()?);
                }
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
    let input = env::args().nth(1);
    let first_launch = info.is_none();
    let enable_debug = cfg!(debug_assertions);
    if enable_debug {
        eprintln!("data {data:?}, info {info:?}");
    }
    // row text
    // let selected = env::args().nth(1);
    let mut stack = Vec::<String>::new();
    if let Some(data) = data {
        for val in data.split('\x03').skip(1) {
            stack.push(val.to_owned());
        }
    }
    let entry = if let Some(info) = info {
        let mut ops: StackOps = info
            .parse()
            .expect("invalid ROFI_INFO (expected stack operation list)");
        if !ops.0.is_empty() {
            for op in ops.0.drain(..ops.0.len() - 1) {
                op.apply(input.as_deref().unwrap_or(""), &mut stack);
            }
            match ops.0.into_iter().next().unwrap() {
                StackOp::Push(s) => {
                    let mut s = s.unwrap_or_else(|| input.unwrap());
                    if enable_debug {
                        eprintln!("using {s}");
                    }
                    if let Some(x) = s.find(';') {
                        let cmd = s.split_off(x + 1);
                        s.pop();
                        let (cmdline, fork) = if let Some(cmd) = cmd.strip_prefix(';') {
                            (cmd, true)
                        } else {
                            (&cmd[..], false)
                        };
                        let mut run = !fork;
                        if fork {
                            if let Ok(Fork::Child) = daemon(true, true) {
                                let _ = nix::unistd::close(stdout().as_raw_fd());
                                run = true;
                            }
                        }
                        if run {
                            let mut cmd = Command::new("bash");
                            cmd.arg("-c");
                            cmd.arg(cmdline);
                            cmd.stdout(Stdio::piped());
                            if let Ok(mut proc) = cmd.spawn() {
                                let mut buf = vec![0u8; 65536];
                                if let Some(stdout) = &mut proc.stdout {
                                    let mut out = stderr().lock();
                                    while let Ok(x) = stdout.read(&mut buf) {
                                        if out.write(&buf[..x]).is_err() {
                                            break;
                                        }
                                    }
                                }
                                let _ = proc.wait();
                            }
                            if fork {
                                let _ = nix::unistd::close(stderr().as_raw_fd());
                                let _ = nix::unistd::close(stdin().as_raw_fd());
                                return;
                            }
                        }
                    }
                    s
                }
                StackOp::Pop => {
                    if enable_debug {
                        eprintln!("quitting!");
                    }
                    return;
                }
            }
        } else {
            if enable_debug {
                eprintln!("using INITIAL_SCRIPT");
            }
            env::var("INITIAL_SCRIPT")
                .expect("INITIAL_SCRIPT must be set as the default submenu to call")
        }
    } else {
        if enable_debug {
            eprintln!("using INITIAL_SCRIPT");
        }
        env::var("INITIAL_SCRIPT")
            .expect("INITIAL_SCRIPT must be set as the default submenu to call")
    };
    let mut cmd = Command::new("bash");
    cmd.arg("--");
    cmd.arg(entry.clone());
    stack.reverse();
    if enable_debug {
        eprintln!("passing args {stack:?}");
    }
    for arg in stack.iter() {
        cmd.arg(arg);
    }
    stack.reverse();
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
        out.write_all(DELIM.as_bytes())
            .expect("failed writing into stdout");
        out.write_all(b"\n").expect("failed writing into stdout");
    }
    let mut opts: ModeOptions = serde_json::from_str(&line).expect("failed to parse menu options");
    for elem in stack {
        opts.data.push('\x03');
        opts.data.push_str(&elem);
    }
    write!(out, "{}", opts.to_rofi()).expect("failed writing menu options into stdout");
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
        match serde_json::from_str::<Row>(line) {
            Ok(mut row) => {
                if row.next_script.is_none() {
                    row.next_script = Some(StackOp::Push(Some(entry.clone())));
                }
                if let Some(row) = row.to_rofi() {
                    if first {
                        first = false;
                    } else {
                        write!(out, "{}", DELIM).expect("failed writing into stdout");
                    }
                    write!(out, "{}", row).expect("failed writing into stdout");
                }
            }
            Err(err) => {
                eprintln!("row parse error ({line}): {err:?}");
            }
        }
    }
}
