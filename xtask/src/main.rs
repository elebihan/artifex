//
// Copyright (C) 2022-2024 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use std::{env, error::Error, fs, os::unix::prelude::PermissionsExt};

use xshell::Shell;

fn install_hooks(shell: &Shell) -> Result<(), Box<dyn Error>> {
    let cwd = shell.current_dir();
    let mut src = cwd.join("hooks");
    src.push("pre-commit.hook");
    let name = src
        .file_stem()
        .ok_or_else(|| "Missing file stem".to_string())?;
    let mut dst = cwd.join(".git");
    dst.push("hooks");
    dst.push(name);
    shell.copy_file(src, &dst)?;
    let mut permissions = fs::metadata(&dst)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(dst, permissions)?;
    Ok(())
}

fn prepare(shell: &Shell) -> Result<(), Box<dyn Error>> {
    install_hooks(shell)
}

fn usage() {
    eprintln!(
        r#"Tasks:

prepare  Prepare development environment
"#
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let shell = Shell::new()?;
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("prepare") => prepare(&shell)?,
        _ => usage(),
    }
    Ok(())
}
