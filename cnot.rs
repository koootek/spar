#![allow(unused, non_camel_case_types)]

#[derive(PartialEq)]
pub enum LogLevel {
    INFO,
    WARNING,
    ERROR,
    NO_LOGS,
}

macro_rules! log {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        let s = format!($fmt, $($arg),*);
        println!("[INFO] {s}");
    };
    ($level:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        let s = format!($fmt, $($arg),*);
        match $level {
            $crate::cnot::LogLevel::INFO    => println!("[INFO] {s}"),
            $crate::cnot::LogLevel::WARNING => println!("[WARN] {s}"),
            $crate::cnot::LogLevel::ERROR   => eprintln!("[ERROR] {s}"),
            _ => {},
        }
    };
}

pub(crate) use log;

pub enum RustEdition {
    R2024,
    R2021,
    R2018,
}

impl std::fmt::Display for RustEdition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(match self {
            Self::R2024 => "2024",
            Self::R2021 => "2021",
            Self::R2018 => "2018",
        })
    }
}

macro_rules! unwrap_bool {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(_) => return true,
        }
    };
}

fn needs_rebuild<T>(output_path: &str, sources: &[T]) -> bool
where 
    T: AsRef<str>,
{
    let output_meta = unwrap_bool!(std::fs::metadata(output_path));

    for source in sources {
        let s = source.as_ref();
        let source_meta = unwrap_bool!(std::fs::metadata(std::path::Path::new(s)));
        let output_time = unwrap_bool!(output_meta.modified());
        let source_time = unwrap_bool!(source_meta.modified());
        if output_time < source_time {
            return true;
        }
    }
    false
}

/// Rebuilds the program with predefined edition (R2024) and O3 optimizations.
///
/// First arg in `proc_args` must be the path to the executable.
///
/// First source file is considered the main file
pub fn rebuild<T>(proc_args: &mut dyn Iterator<Item = String>, sources: &[T])
where
    T: AsRef<str>,
{
    rebuild_edition(proc_args, RustEdition::R2024, sources);
}

/// Rebuilds the program with O3 optimizations and a custom edition.
///
/// First arg in `proc_args` must be the path to the executable.
///
/// First source file is considered the main file
pub fn rebuild_edition<T>(
    proc_args: &mut dyn Iterator<Item = String>,
    edition: RustEdition,
    sources: &[T],
) where
    T: AsRef<str>,
{
    rebuild_edition_args(
        proc_args,
        edition,
        sources,
        &[("-O", None)],
    );
}

/// Rebuilds the program with no additional flags and a custom edition.
///
/// First arg in `proc_args` must be the path to the executable.
///
/// First source file is considered the main file
pub fn rebuild_edition_args<T>(
    proc_args: &mut dyn Iterator<Item = String>,
    edition: RustEdition,
    sources: &[T],
    rustc_args: &[(&str, Option<&str>)],
) where
    T: AsRef<str>,
{
    let self_path = match proc_args.next() {
        Some(self_path) => self_path,
        None => return,
    };
    if !needs_rebuild(&self_path, &sources) {
        return;
    }

    let mut args = vec![];
    for (arg, value) in rustc_args {
        args.push(arg);
        if let Some(value) = value {
            args.push(value);
        }
    }

    let status = std::process::Command::new("rustc")
        .args(args)
        .args([
            "--edition",
            &edition.to_string(),
            "-o",
            &self_path,
            sources[0].as_ref()
        ])
        .status()
        .expect("failed to rebuild");

    if !status.success() {
        log!(LogLevel::ERROR, "Build failed");
        std::process::exit(1);
    }

    log!(LogLevel::INFO, "Build successful");
    std::process::Command::new(&self_path)
        .args(proc_args)
        .spawn()
        .expect("program failed to run")
        .wait()
        .expect("program did not run");
    std::process::exit(0);
}

/// Generates `rust-project.json` to fix rust-analyzer not working on standalone files.
pub fn generate_project(root_file: &str, edition: RustEdition) -> std::io::Result<()> {
    if std::fs::exists("rust-project.json")? {
        return Ok(());
    }

    let sysroot_path = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .expect("failed to get sysroot");
    if !sysroot_path.status.success() {
        eprintln!("Failed to get sysroot path");
        return Ok(());
    }
    let sysroot_path = String::from_utf8(sysroot_path.stdout).unwrap();
    let mut sysroot_path = sysroot_path.lines();

    std::fs::write(
        "rust-project.json",
        &format!(
            r#"{{
"sysroot_src": "{}/lib/rustlib/src/rust/library",
"crates": [
    {{
        "root_module": "{}",
        "edition": "{}",
        "deps": []
    }}
]
}}"#,
            sysroot_path.next().ok_or_else(|| std::io::Error::other("failed to get sysroot path"))?,
            root_file,
            edition
        ),
    )?;
    Ok(())
}
