//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use crate::error::Result;
use nix::sys::{sysinfo::sysinfo, utsname::uname};
use std::time::Duration;

pub struct MachineInfo {
    pub kernel_version: String,
    pub system_uptime: Duration,
}

pub fn get_machine_info() -> Result<MachineInfo> {
    let kernel_version = uname().map(|u| u.release().to_string_lossy().to_string())?;
    let system_uptime = sysinfo().map(|i| i.uptime())?;
    Ok(MachineInfo {
        kernel_version,
        system_uptime,
    })
}
