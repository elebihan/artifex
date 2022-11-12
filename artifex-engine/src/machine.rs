//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use crate::error::{Error, Result};

// From https://github.com/GuillaumeGomez/sysinfo/blob/master/src/linux/system.rs
fn get_kernel_version() -> Option<String> {
    let mut raw = std::mem::MaybeUninit::<libc::utsname>::zeroed();

    unsafe {
        if libc::uname(raw.as_mut_ptr()) == 0 {
            let info = raw.assume_init();
            let release = info
                .release
                .iter()
                .filter(|c| **c != 0)
                .map(|c| *c as u8 as char)
                .collect::<String>();
            Some(release)
        } else {
            None
        }
    }
}

pub struct MachineInfo {
    pub kernel_version: String,
}

pub fn get_machine_info() -> Result<MachineInfo> {
    let kernel_version = get_kernel_version().ok_or(Error::Unknown)?;
    Ok(MachineInfo { kernel_version })
}
