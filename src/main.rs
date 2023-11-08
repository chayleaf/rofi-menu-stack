use fork::Fork;
use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};
use std::{
    env,
    io::{stdout, BufRead, BufReader, Write},
    process::{Command, Stdio},
};

mod fallback_row;
mod options;
mod row;

use options::ModeOptions;
use row::Row;

const DELIM: char = '\x0b';

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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Data {
    pub stack: Vec<String>,
    pub call_stack: Vec<String>,
    pub fallback: Option<Info>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Info {
    pub push_call: VecString,
    pub push: VecString,
    pub pop_call: Option<usize>,
    pub pop: Option<usize>,
    pub exec: VecString,
    pub fork: bool,
    pub menu: Option<Box<ModeOptions>>,
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
            menu: None,
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
    let mut out = stdout().lock();
    if first_launch {
        if let Some(input) = input {
            #[allow(clippy::single_match)]
            match input.as_str() {
                "unjson5" => {
                    out.write_all(
                        &serde_json::to_vec(
                            &json5::from_str::<serde_json::Value>(
                                &env::args()
                                    .nth(2)
                                    .expect("provide json5 to convert to json"),
                            )
                            .expect("invalid json5"),
                        )
                        .expect("failed to serialize json"),
                    )
                    .expect("failed writing into stdout");
                }
                _ => {}
            }
            return;
        } else {
            out.write_all(b"\0delim\x1F")
                .expect("failed writing into stdout");
            out.write_all(&[DELIM as u8])
                .expect("failed writing into stdout");
            out.write_all(b"\n").expect("failed writing into stdout");
        }
    }
    let enable_debug = cfg!(debug_assertions);
    if enable_debug {
        eprintln!("data {data:?}, info {info:?}");
    }
    let mut data: Data = json5::from_str(&data.unwrap_or_default()).unwrap_or_default();
    let mut info: Info = info.as_deref().map_or_else(
        || data.fallback.clone().unwrap_or_default(),
        |info| json5::from_str(info).expect("failed to parse info"),
    );
    let mut input = input.as_deref().unwrap_or_default().to_owned();
    loop {
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
                cmd.arg("\"$0\" \"$@\"").args(info.exec.flatten1(&input));
            } else {
                cmd.arg(info.exec.flatten(&input));
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
        for x in info.push.flatten1(&input) {
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
        for x in info.push_call.flatten1(&input) {
            data.call_stack.push(x);
        }
        if enable_debug {
            eprintln!("data {data:?}, info {info:?}");
        }
        let Some(argv0) = data.call_stack.last() else {
            return;
        };
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg("\"$0\" \"$@\"")
            .arg(argv0)
            .env("_CALL_STACK_LEN", (data.call_stack.len() - 1).to_string());
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
        buf.read_line(&mut line)
            .expect("failed to read menu options");
        if enable_debug {
            eprintln!("opts: {line:?}");
        }
        let mut opts: ModeOptions = json5::from_str(&line).expect("failed to parse menu options");
        opts.data.call_stack = data.call_stack.clone();
        opts.data.stack = data.stack.clone();
        if let Some(menu) = &info.menu {
            opts.merge(menu);
        }
        let mut first = true;
        let mut first_row = None;
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
                    if first {
                        if row.to_rofi().is_some() {
                            first_row = Some(row);
                            first = false;
                        }
                    } else if let Some(row) = row.to_rofi() {
                        if let Some(row) = first_row.take() {
                            out.write_all(opts.to_rofi().as_bytes())
                                .expect("failed writing menu options into stdout");
                            out.write_all(row.to_rofi().unwrap().as_bytes())
                                .expect("failed writing into stdout");
                        }
                        out.write_all(&[DELIM as u8])
                            .expect("failed writing into stdout");
                        out.write_all(row.as_bytes())
                            .expect("failed writing into stdout");
                    }
                }
                Err(err) => {
                    eprintln!("row parse error ({line}):\n{err}");
                }
            }
        }
        if let Some(row) = first_row.take() {
            if opts.autoselect {
                info = row.info;
                data = opts.data;
                input = row.text;
                continue;
            } else {
                out.write_all(opts.to_rofi().as_bytes())
                    .expect("failed writing menu options into stdout");
                out.write_all(row.to_rofi().unwrap().as_bytes())
                    .expect("failed writing into stdout");
            }
        }
        break;
    }
}
