use fork::{daemon, Fork};
use serde::{de::Visitor, Deserialize};
use std::{
    env,
    io::{stderr, stdin, stdout, BufRead, BufReader, Read, Write},
    os::fd::AsRawFd,
    process::{Command, Stdio},
};

mod row;
mod row2;
mod stack_op;
mod stack_op2;

use row::*;
use row2::*;
use stack_op::*;
use stack_op2::*;

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
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Set(v as i64))
    }
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Selection::Set(v as i64))
    }
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
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
    /// Selection mode
    selection: Selection,
    /// Next ROFI_DATA
    data: String,
    /// Fallback for freeform text input
    fallback: Option<FallbackRow>,
}

impl Default for ModeOptions {
    fn default() -> Self {
        Self {
            prompt: "".to_owned(),
            message: "".to_owned(),
            markup: Markup::None,
            selection: Selection::Reset,
            data: "".to_owned(),
            fallback: None,
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
        if self.fallback.is_none() {
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
                "fallback" => ret.fallback = map.next_value()?,
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

fn main() {
    // 0: init
    // 1: selected entry
    // 2: selected custom entry
    // 10-28: custom keybindings 1-19
    // let retv = env::var("ROFI_RETV").ok();
    // common info
    let data = env::var("ROFI_DATA").ok();
    // row info
    let mut info = env::var("ROFI_INFO").ok();
    let input = env::args().nth(1);
    let first_launch = info.is_none() && data.is_none();
    let enable_debug = cfg!(debug_assertions);
    if enable_debug {
        eprintln!("data {data:?}, info {info:?}");
    }
    // row text
    // let selected = env::args().nth(1);
    let mut stack = Vec::<String>::new();
    if let Some(data) = data {
        let mut it = data.split('\x04');
        if let Some(fallback) = it.next() {
            for val in it {
                stack.push(val.to_owned());
            }
            if info.is_none() {
                info = Some(fallback.to_owned());
                if enable_debug {
                    eprintln!("new info {info:?}");
                }
            }
        }
    }
    let input = input.as_deref().unwrap_or_default();
    let entry = if let Some(info) = info {
        let mut ops: FallbackStackOps = info
            .parse()
            .expect("invalid ROFI_INFO (expected stack operation list)");
        if !ops.0.is_empty() {
            for op in ops.0.drain(..ops.0.len() - 1) {
                op.apply(input, &mut stack);
            }
            match ops.0.into_iter().next().unwrap() {
                x @ (FallbackStackOp::Push(_) | FallbackStackOp::PushUser) => {
                    let mut s = match x {
                        FallbackStackOp::Push(s) => s,
                        _ => input.to_owned(),
                    };
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
                FallbackStackOp::Pop => {
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
    if let Some(fallback) = &mut opts.fallback {
        if fallback.next_script.is_none() {
            fallback.next_script = Some(FallbackStackOp::Push(entry.clone()));
        }
        opts.data.push_str(&fallback.info());
    }
    for elem in stack {
        opts.data.push('\x04');
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
                    row.next_script = Some(StackOp::Push(entry.clone()));
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
