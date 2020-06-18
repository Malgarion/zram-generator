/* SPDX-License-Identifier: MIT */

mod config;
mod generator;
mod setup;

use anyhow::Result;
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};
use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Opts {
    /// Generate units into the directory
    GenerateUnits(String),
    /// Set up a single device
    SetupDevice(String),
    /// Reset (destroy) a device
    ResetDevice(String),
}

fn get_opts() -> Opts {
    #[allow(deprecated)]
    let opts = app_from_crate!("\n")
        .arg(
            Arg::from_usage("--setup-device 'Set up a single device'")
                .conflicts_with("reset-device"),
        )
        .arg(Arg::from_usage("--reset-device 'Reset (destroy) a device'"))
        .arg(Arg::from_usage(
            "<directory/device> 'Target directory for generator or device to operate on'",
        ))
        .arg(
            Arg::from_usage(
                "[extra-dir] 'Unused target directories to satisfy systemd.generator(5)'",
            )
            .number_of_values(2)
            .conflicts_with_all(&["setup-device", "reset-device"]),
        )
        .get_matches();

    let val = opts
        .value_of("directory/device")
        .expect("clap invariant")
        .to_string();
    if opts.is_present("setup-device") {
        Opts::SetupDevice(val)
    } else if opts.is_present("reset-device") {
        Opts::ResetDevice(val)
    } else {
        Opts::GenerateUnits(val)
    }
}

fn main() -> Result<()> {
    let mut have_env_var = false;

    let root: Cow<'static, str> = match env::var("ZRAM_GENERATOR_ROOT") {
        Ok(val) => {
            have_env_var = true;
            val.into()
        }
        Err(env::VarError::NotPresent) => "/".into(),
        Err(e) => return Err(e.into()),
    };
    let root = Path::new(&root[..]);

    match get_opts() {
        Opts::GenerateUnits(target) => {
            let devices = config::read_all_devices(&root)?;
            let output_directory = PathBuf::from(target);
            generator::run_generator(&devices, &output_directory, have_env_var)
        }
        Opts::SetupDevice(dev) => {
            let device = config::read_device(&root, &dev)?;
            setup::run_device_setup(device, &dev)
        }
        Opts::ResetDevice(dev) => {
            // We don't read the config here, so that it's possible to remove a device
            // even after the config has been removed.
            setup::run_device_reset(&dev)
        }
    }
}
