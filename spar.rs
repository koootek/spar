#![allow(unused)]

use std::{cell::RefCell, rc::Rc};

const FLAG_CAP: usize = 256;

thread_local! {
    static FLAGS: RefCell<FlagContext> = RefCell::new(FlagContext::new());
}

pub struct FlagContext {
    flags: [Option<Rc<RefCell<OwnedFlag>>>; FLAG_CAP],
    position: usize,
    flag_ignore: bool,
    assign_short_form: bool,
}

impl FlagContext {
    const fn new() -> Self {
        Self {
            flags: [const { None }; FLAG_CAP],
            position: 0,
            flag_ignore: true,
            assign_short_form: true,
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
    short_form: &'static str,
    value: FlagValue,
}

impl OwnedFlag {
    fn new(name: &'static str, value: FlagValue) -> Self {
        Self {
            name, short_form: "", value
        }
    }

    fn with_short_form(name: &'static str, short_form: &'static str, value: FlagValue) -> Self {
        Self {
            name, short_form, value
        }
    }

    fn empty() -> Self {
        Self {
            name: "", short_form: "", value: FlagValue::Empty,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn short_form(&self) -> &str {
        self.short_form
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

impl FlagValue {
    fn parse(&mut self, msg: String) -> Result<(), SparError> {
        match self {
            FlagValue::Long(value) => *value = msg.parse()?,
            FlagValue::ULong(value) => *value = msg.parse()?,
            FlagValue::Float(value) => *value = msg.parse()?,
            FlagValue::Double(value) => *value = msg.parse()?,
            FlagValue::String(value) => {
                if msg.starts_with("\"") {
                    if msg.len() <= 1 {
                        return Err(SparError::ParseString(msg))
                    }
                    *value = msg[1..msg.len() - 1].to_string();
                } else {
                    *value = msg;
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
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

#[derive(Debug)]
pub enum SparError {
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    ParseString(String),
    NoMoreArgs,
}

impl From<std::num::ParseIntError> for SparError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<std::num::ParseFloatError> for SparError {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::ParseFloat(value)
    }
}

pub fn parse_args(proc_args: &mut dyn Iterator<Item = String>) -> Result<(), SparError> {
    FLAGS.with_borrow_mut(|ctx| -> Result<(), SparError> {
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
            let ignore = ctx.flag_ignore && ignore;
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
                if flag.name != &name && flag.short_form != &name {
                    continue;
                }

                if ignore {
                    match &flag.value {
                        FlagValue::Bool(_) => {},
                        FlagValue::Long(_)
                            | FlagValue::ULong(_) | FlagValue::Float(_)
                            | FlagValue::Double(_) | FlagValue::String(_) => {
                            proc_args.next().ok_or(SparError::NoMoreArgs)?;
                        }
                        FlagValue::Empty => unreachable!(),
                    }
                    break;
                }

                let arg = match &mut flag.value {
                    FlagValue::Bool(value) => {
                        *value = !*value;
                        break;
                    }
                    _ => proc_args.next().ok_or(SparError::NoMoreArgs)?,
                    FlagValue::Empty => unreachable!(),
                };

                flag.value.parse(arg)?;
                break;
            }
        }
        Ok(())
    })
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

    pub fn short_form(&self) -> &str {
        self.get().short_form
    }

    pub fn value(&self) -> FlagValue {
        self.get().value.clone()
    }
}

pub fn disable_flag_ignore() {
    FLAGS.with_borrow_mut(|ctx| ctx.flag_ignore = false);
}

pub fn disable_assign_short_form() {
    FLAGS.with_borrow_mut(|ctx| ctx.assign_short_form = false);
}

fn new_flag(name: &'static str, value: FlagValue) -> Flag {
    FLAGS.with_borrow_mut(|ctx| {
        if ctx.position == FLAG_CAP {
            panic!("exceeded FLAG_CAP={}", FLAG_CAP);
        }
        let flag = Rc::new(RefCell::new(
                OwnedFlag::with_short_form(
                    name,
                    if ctx.assign_short_form { &name[0..=0] } else { "" },
                    value
                )));
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

fn new_flag_short(name: &'static str, short_form: &'static str, value: FlagValue) -> Flag {
    FLAGS.with_borrow_mut(|ctx| {
        if ctx.position == FLAG_CAP {
            panic!("exceeded FLAG_CAP={}", FLAG_CAP);
        }
        let flag = Rc::new(RefCell::new(OwnedFlag::with_short_form(name, short_form, value)));
        ctx.push(Rc::clone(&flag));
        Flag::new(flag)
    })
}

/// Create a new boolean flag with short form
///
/// This flag works like a toggle, i.e. value = !value
pub fn flag_bool_short(name: &'static str, short_form: &'static str, default_value: bool) -> Flag {
    new_flag_short(name, short_form, FlagValue::Bool(default_value))
}

/// Create a new long flag with short form
pub fn flag_long_short(name: &'static str, short_form: &'static str, default_value: i64) -> Flag {
    new_flag_short(name, short_form, FlagValue::Long(default_value))
}

/// Create a new ulong flag with short form
pub fn flag_ulong_short(name: &'static str, short_form: &'static str, default_value: u64) -> Flag {
    new_flag_short(name, short_form, FlagValue::ULong(default_value))
}

/// Create a new float flag with short form
pub fn flag_float_short(name: &'static str, short_form: &'static str, default_value: f32) -> Flag {
    new_flag_short(name, short_form, FlagValue::Float(default_value))
}

/// Create a new double flag with short form
pub fn flag_double_short(name: &'static str, short_form: &'static str, default_value: f64) -> Flag {
    new_flag_short(name, short_form, FlagValue::Double(default_value))
}

/// Create a new string flag with short form
///
/// Accepted input values:
/// - content
/// - "content"
pub fn flag_string_short(name: &'static str, short_form: &'static str, default_value: &str) -> Flag {
    new_flag_short(name, short_form, FlagValue::String(default_value.to_string()))
}
