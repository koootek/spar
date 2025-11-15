use std::sync::Mutex;

static FLAGS: Mutex<Vec<Flag>> = Mutex::new(Vec::new());

#[derive(Debug)]
pub struct Flag {
    name: &'static str,
    value: FlagValue,
}

impl Flag {
    fn new(name: &'static str, value: FlagValue) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn value(&self) -> &FlagValue {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FlagValue {
    Bool(bool),
    Long(i64),
    ULong(u64),
    Float(f32),
    Double(f64),
    String(String),
}

impl std::fmt::Display for FlagValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::Long(value) => write!(f, "{value}"),
            Self::ULong(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::Double(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
        }
    }
}

pub fn parse_args(proc_args: &mut dyn Iterator<Item = String>) {
    let mut flags = FLAGS.lock().unwrap();
    while let Some(arg) = proc_args.next() {
        if arg.len() < 2 {
            continue;
        }

        let mut chars = arg.chars().peekable();
        if chars.next().unwrap() != '-' {
            continue;
        }

        let ignore = *chars.peek().unwrap() == '/';
        let mut name = String::new();
        if ignore {
            chars.next().unwrap();
        }
        for c in chars {
            name.push(c);
        }

        for flag in flags.iter_mut() {
            if flag.name != &name {
                continue;
            }

            match &mut flag.value {
                FlagValue::Bool(value) => {
                    if !ignore {
                        *value = !*value;
                    }
                }
                FlagValue::Long(value) => {
                    let arg = proc_args.next().unwrap();
                    if !ignore {
                        *value = arg.parse().unwrap();
                    }
                }
                FlagValue::ULong(value) => {
                    let arg = proc_args.next().unwrap();
                    if !ignore {
                        *value = arg.parse().unwrap();
                    }
                }
                FlagValue::Float(value) => {
                    let arg = proc_args.next().unwrap();
                    if !ignore {
                        *value = arg.parse().unwrap();
                    }
                }
                FlagValue::Double(value) => {
                    let arg = proc_args.next().unwrap();
                    if !ignore {
                        *value = arg.parse().unwrap();
                    }
                }
                FlagValue::String(value) => {
                    let arg = proc_args.next().unwrap();
                    if ignore {
                        continue;
                    }

                    if arg.starts_with("\"") {
                        *value = arg[1..arg.len() - 1].to_string();
                    } else {
                        *value = arg;
                    }
                }
            }
        }
    }
}

fn new_flag(name: &'static str, value: FlagValue) -> &'static Flag {
    let mut flags = FLAGS.lock().unwrap();
    flags.push(Flag::new(name, value));
    let ptr = flags.last().unwrap() as *const _;
    unsafe { &*ptr }
}

/// Create a new boolean flag
///
/// This flag works like a toggle, i.e. value = !value
pub fn flag_bool(name: &'static str, default_value: bool) -> &'static Flag {
    new_flag(name, FlagValue::Bool(default_value))
}

/// Create a new long flag
pub fn flag_long(name: &'static str, default_value: i64) -> &'static Flag {
    new_flag(name, FlagValue::Long(default_value))
}

/// Create a new ulong flag
pub fn flag_ulong(name: &'static str, default_value: u64) -> &'static Flag {
    new_flag(name, FlagValue::ULong(default_value))
}

/// Create a new float flag
pub fn flag_float(name: &'static str, default_value: f32) -> &'static Flag {
    new_flag(name, FlagValue::Float(default_value))
}

/// Create a new double flag
pub fn flag_double(name: &'static str, default_value: f64) -> &'static Flag {
    new_flag(name, FlagValue::Double(default_value))
}

/// Create a new string flag
///
/// Accepted input values:
/// - content
/// - "content"
pub fn flag_string(name: &'static str, default_value: &str) -> &'static Flag {
    new_flag(name, FlagValue::String(default_value.to_string()))
}
