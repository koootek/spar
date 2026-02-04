#![allow(unused)]

use std::{cell::RefCell, rc::Rc};

const FLAG_CAP: usize = 256;

thread_local! {
    static FLAGS: RefCell<FlagContext> = RefCell::new(FlagContext::new());
}

pub struct FlagContext {
    flags: [Option<Rc<RefCell<OwnedFlag>>>; FLAG_CAP],
    position: usize,
}

impl FlagContext {
    const fn new() -> Self {
        Self {
            flags: [const { None }; FLAG_CAP],
            position: 0
        }
    }

    fn push(&mut self, flag: Rc<RefCell<OwnedFlag>>) {
        self.flags[self.position] = Some(flag);
        self.position += 1;
    }
}

#[derive(Debug, Clone)]
pub struct OwnedFlag {
    name: &'static str,
    value: FlagValue,
}

impl OwnedFlag {
    fn new(name: &'static str, value: FlagValue) -> Self {
        Self { name, value }
    }

    fn empty() -> Self {
        Self {
            name: "", value: FlagValue::Empty,
        }
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
    Empty,
}

impl std::fmt::Display for FlagValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Bool(value)   => write!(f, "{value}"),
            Self::Long(value)   => write!(f, "{value}"),
            Self::ULong(value)  => write!(f, "{value}"),
            Self::Float(value)  => write!(f, "{value}"),
            Self::Double(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
            Self::Empty         => unreachable!(),
        }
    }
}

pub fn parse_args(proc_args: &mut dyn Iterator<Item = String>) {
    FLAGS.with_borrow_mut(|ctx| {
        while let Some(arg) = proc_args.next() {
            let mut chars = arg.chars().peekable();
            while let Some(ch) = chars.peek() {
                if *ch != '-' {
                    break;
                }

                chars.next();
            }

            let ignore = match chars.peek() {
                Some(ch) => *ch == '/',
                None => continue,
            };
            if ignore {
                chars.next();
            }
            let mut name = String::new();
            for c in chars {
                name.push(c);
            }

            for flag in ctx.flags.iter_mut() {
                if flag.is_none() {
                    break;
                }
                let flag = flag.as_mut().unwrap();
                let mut flag = flag.borrow_mut();
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
                    FlagValue::Empty => unreachable!(),
                }
                break;
            }
        }
    });
}

pub struct Flag {
    inner: Rc<RefCell<OwnedFlag>>,
}

impl Flag {
    fn new(value: Rc<RefCell<OwnedFlag>>) -> Self {
        Self {
            inner: value,
        }
    }

    #[inline]
    fn get(&self) -> std::cell::Ref<'_, OwnedFlag> {
        self.inner.borrow()
    }

    pub fn name(&self) -> &str {
        self.get().name
    }

    pub fn value(&self) -> FlagValue {
        self.get().value.clone()
    }
}

fn new_flag(name: &'static str, value: FlagValue) -> Flag {
    FLAGS.with_borrow_mut(|ctx| {
        if ctx.position == FLAG_CAP {
            panic!("exceeded FLAG_CAP={}", FLAG_CAP);
        }
        let flag = Rc::new(RefCell::new(OwnedFlag::new(name, value)));
        ctx.push(Rc::clone(&flag));
        Flag::new(flag)
    })
}

/// Create a new boolean flag
///
/// This flag works like a toggle, i.e. value = !value
pub fn flag_bool(name: &'static str, default_value: bool) -> Flag {
    new_flag(name, FlagValue::Bool(default_value))
}

/// Create a new long flag
pub fn flag_long(name: &'static str, default_value: i64) -> Flag {
    new_flag(name, FlagValue::Long(default_value))
}

/// Create a new ulong flag
pub fn flag_ulong(name: &'static str, default_value: u64) -> Flag {
    new_flag(name, FlagValue::ULong(default_value))
}

/// Create a new float flag
pub fn flag_float(name: &'static str, default_value: f32) -> Flag {
    new_flag(name, FlagValue::Float(default_value))
}

/// Create a new double flag
pub fn flag_double(name: &'static str, default_value: f64) -> Flag {
    new_flag(name, FlagValue::Double(default_value))
}

/// Create a new string flag
///
/// Accepted input values:
/// - content
/// - "content"
pub fn flag_string(name: &'static str, default_value: &str) -> Flag {
    new_flag(name, FlagValue::String(default_value.to_string()))
}
