use std::env::{current_dir, var};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, format_err, Result};
use structopt::StructOpt;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        eprintln!("{}: {}", env!("CARGO_PKG_NAME"), format_args!($($arg)*));
    })
}

#[derive(StructOpt)]
pub struct CommonOpt {
    #[structopt(long, short)]
    pub quiet: bool,
    #[structopt(long, short)]
    pub force: bool,
    pub version: Option<String>,
}

fn nonempty(s: Option<String>) -> Option<String> {
    if let Some(s) = s {
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    } else {
        None
    }
}

fn pyenv_root() -> Result<PathBuf> {
    let root = nonempty(var("PYENV_ROOT").ok())
        .ok_or_else(|| format_err!("env `PYENV_ROOT` not found"))?;
    Ok(PathBuf::from(root))
}

fn pyenv_version_file() -> Result<PathBuf> {
    let cwd = current_dir()?;
    for dir in cwd.ancestors() {
        let path = dir.join(".python-version");
        if path.exists() {
            return Ok(path);
        }
    }
    Ok(pyenv_root()?.join("version"))
}

fn pyenv_version_file_read(file: &Path) -> Result<String> {
    let mut file = File::open(file)?;
    let mut buf = vec![0; 1024];
    let len = file.read(&mut buf)?;
    buf.truncate(len);
    let version: Vec<u8> = buf
        .into_iter()
        .skip_while(u8::is_ascii_whitespace)
        .take_while(|b| !b.is_ascii_whitespace())
        .collect();
    Ok(String::from_utf8(version)?)
}

fn pyenv_version_name() -> String {
    let version = nonempty(var("PYENV_VERSION").ok()).or_else(|| {
        pyenv_version_file()
            .and_then(|f| pyenv_version_file_read(&f))
            .ok()
    });
    if version.is_none() {
        return "system".to_string();
    }
    // TODO: check installed
    version.unwrap()
}

fn pyenv_prefix(version: &str) -> Result<PathBuf> {
    if version == "system" {
        todo!()
    } else {
        let dir = pyenv_root()?.join("versions").join(version);
        if dir.is_dir() {
            Ok(dir.canonicalize()?)
        } else {
            Err(format_err!("version `{}` not installed", version))
        }
    }
}

pub fn pyenv_sh_deactivate(force: bool, quiet: bool) -> Result<()> {
    let virtual_env = nonempty(var("VIRTUAL_ENV").ok());
    if virtual_env.is_none() && !force {
        if !quiet {
            log!("no virtualenv has been activated.");
        }
        bail!("no virtualenv has been activated.");
    }

    if nonempty(var("PYENV_ACTIVATE_SHELL").ok()).is_some() {
        println!("unset PYENV_VERSION;");
        println!("unset PYENV_ACTIVATE_SHELL;");
    }

    println!("unset PYENV_VIRTUAL_ENV;");
    println!("unset VIRTUAL_ENV;");

    println!(
        r#"if [ -n "${{_OLD_VIRTUAL_PATH}}" ]; then
  export PATH="${{_OLD_VIRTUAL_PATH}}";
  unset _OLD_VIRTUAL_PATH;
fi;"#
    );

    println!(
        r#"if [ -n "${{_OLD_VIRTUAL_PYTHONHOME}}" ]; then
  export PYTHONHOME="${{_OLD_VIRTUAL_PYTHONHOME}}";
  unset _OLD_VIRTUAL_PYTHONHOME;
fi;"#
    );

    println!(
        r#"if declare -f deactivate 1>/dev/null 2>&1; then
  unset -f deactivate;
fi;"#
    );

    Ok(())
}

pub fn pyenv_sh_activate(version: Option<String>, force: bool, quiet: bool) -> Result<()> {
    let version = nonempty(version).unwrap_or_else(pyenv_version_name);

    let virtual_env = nonempty(var("VIRTUAL_ENV").ok());
    if let Some(virtual_env) = &virtual_env {
        if nonempty(var("PYENV_VIRTUAL_ENV").ok()).is_none() && !force {
            if !quiet {
                log!("virtualenv `{}` is already activated", virtual_env);
            }
            println!("true");
            return Ok(());
        }
    }

    ensure!(
        version != "system",
        "version `{}` is not a virtualenv",
        version
    );

    let prefix = pyenv_prefix(&version)?;
    ensure!(
        prefix.join("bin").join("python").is_file(),
        "`python` not found in version `{}`",
        version
    );
    ensure!(
        prefix.join("bin").join("activate").is_file(),
        "version `{}` is not a virtualenv",
        version
    );

    if virtual_env.as_ref().map(String::as_ref) == Some(prefix.to_string_lossy().as_ref()) && !force
    {
        if !quiet {
            log!("version `{}` is already activated", version);
        }
        println!("true");
        return Ok(());
    }

    let _ = pyenv_sh_deactivate(true, true);

    println!(r#"export PYENV_VERSION="{}";"#, version);
    println!("export PYENV_ACTIVATE_SHELL=1;");

    println!(r#"export PYENV_VIRTUAL_ENV="{}";"#, prefix.display());
    println!(r#"export VIRTUAL_ENV="{}";"#, prefix.display());

    if let Some(pythonhome) = nonempty(var("PYTHONHOME").ok()) {
        println!(r#"export _OLD_VIRTUAL_PYTHONHOME="{}";"#, pythonhome);
        println!(r#"unset PYTHONHOME;"#);
    }

    Ok(())
}

pub fn handle_result(res: Result<()>, quiet: bool) {
    if let Err(e) = res {
        if !quiet {
            log!("{}", e);
        }
        println!("false");
        std::process::exit(1);
    }
}
