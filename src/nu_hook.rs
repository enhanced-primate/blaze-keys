use crate::{
    CONFIG_DIR, NU_SOURCE_NAME,
    keys::{self, NuKey},
    yml::GlobalConfig,
};
use anyhow::{Context, Result};
use log::debug;
use std::io::Write as write2;
use std::{fmt::Write, fs::File};

pub fn nu_source_location() -> String {
    CONFIG_DIR.join(NU_SOURCE_NAME).to_str().unwrap().into()
}

fn write_to_file(content: &str) -> anyhow::Result<()> {
    if !CONFIG_DIR.exists() {
        std::fs::create_dir_all(CONFIG_DIR.to_str().unwrap())?;
    }
    let mut f = File::create(nu_source_location())?;

    debug!("Writing {content:?} to {f:?}");

    write!(&mut f, "{}", content)
        .context("Failed to generate nu file which contains keybindings for leader keys")
}

/// Generate the file containing the code which adds the nushell keybindings to trigger leader keys.
pub fn generate_nu_source(global: &Option<GlobalConfig>) -> Result<()> {
    let mut buffer = String::new();

    if let Some(g) = global.as_ref().and_then(|g| g.global.as_ref()) {
        if let Some(ref leaders) = g.leader_keys {
            buffer.reserve(1000 * 2 * leaders.len());

            if !leaders.is_empty() {
                writeln!(
                    &mut buffer,
                    "##### blaze-keys: start\n##### The nu widgets which provide the leader key functionality."
                )?;
            }

            for leader in leaders.iter() {
                for (_i, k) in [&leader.exec_mode, &leader.abbr_mode].iter().enumerate() {
                    let abbr = _i == 1;
                    let key = keys::get_key_nu(k);

                    let NuKey { modifier, char } = match &key {
                        Some(k) => k,
                        _ => panic!("invalid keybind for leader"),
                    };
                    let modifier = match modifier {
                        None => anyhow::bail!("Need a modifier key for a leader key trigger"),
                        Some(m) => m,
                    };
                    let (flag, spacing, accept_flag) = match abbr {
                        true => ("--porcelain-abbr ", "commandline edit --insert ' ';", ""),
                        false => ("", "", "-A"),
                    };

                    writeln!(&mut buffer,
                        "
$env.config.keybindings ++= [
    {{
      name: blz_{0}
      modifier: {2}
      keycode: {3}
      mode: emacs
      event: {{
        send: executehostcommand,
        cmd: \"let tmpfile = (mktemp -p /tmp); blz --porcelain-leader {0} {5} --porcelain-tmp $tmpfile; commandline edit {1} --insert (cat $tmpfile);{4} rm $tmpfile\"
      }} 
    }}
]
",
                        leader.sanitized_name(),
                        accept_flag,
                        modifier,
                        char,
                        spacing,
                        flag,
                    )?;
                }
            }
        }
        writeln!(&mut buffer, "##### blaze-keys: end")?;
    }

    write_to_file(&buffer)
}
