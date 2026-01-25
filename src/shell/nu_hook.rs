use super::leaders_to_state;
use crate::{
    CONFIG_DIR, NU_SOURCE_NAME,
    keys::{self, NuKey},
    yml::GlobalConfig,
};
use anyhow::{Context, Result};
use log::debug;
use std::io::{BufRead, BufReader, Write as write2};
use std::{fmt::Write, fs::File};

const BLZ_LEADER_PREFIX: &str = "##### BLZ_LEADER_STATE: ";

pub fn nu_source_location() -> String {
    CONFIG_DIR.join(NU_SOURCE_NAME).to_str().unwrap().into()
}

fn read_leader_state_from_file() -> Option<String> {
    let f = File::open(nu_source_location()).ok()?;
    let b = BufReader::new(f);

    match b.lines().nth(3) {
        Some(ld_line) => ld_line.map(|value| parse_leader_state(&value)).ok(),
        None => None,
    }
    .flatten()
}

fn parse_leader_state(value: &str) -> Option<String> {
    if value.starts_with(BLZ_LEADER_PREFIX) {
        Some(value.split_at(BLZ_LEADER_PREFIX.len()).1.to_owned())
    } else {
        None
    }
}

#[test]
fn test_parse_leader_state_from_nu_source() {
    assert_eq!(
        Some("tenpins".to_string()),
        parse_leader_state("##### BLZ_LEADER_STATE: tenpins")
    );
}

fn write_to_file(content: &str) -> anyhow::Result<()> {
    if !CONFIG_DIR.exists() {
        std::fs::create_dir_all(CONFIG_DIR.to_str().unwrap())?;
    }
    let mut f = File::create(nu_source_location())?;

    debug!("Writing {content:?} to {f:?}");

    write!(&mut f, "{}", content)
        .context("Failed to generate nu file which contains keybindings for leader keys")?;

    f.flush()?;
    Ok(())
}

/// Generate the file containing the code which adds the nushell keybindings to trigger leader keys.
pub fn generate_nu_source(global: &Option<GlobalConfig>) -> Result<()> {
    let mut buffer = String::new();

    if let Some(g) = global.as_ref().and_then(|g| g.global.as_ref()) {
        let leader_state = leaders_to_state(&g.leader_keys.as_ref());

        match read_leader_state_from_file() {
            Some(extant) if extant == leader_state => {
                debug!("No need to rewrite the nu source file");
                return Ok(());
            }
            _ => (),
        };

        if let Some(ref leaders) = g.leader_keys {
            buffer.reserve(830 * leaders.len());

            if !leaders.is_empty() {
                writeln!(
                    &mut buffer,
                    "##### blaze-keys: start v1\n##### The nu widgets which provide the leader key functionality.\n"
                )?;
            }

            // Must be on 4th line.
            writeln!(&mut buffer, "{BLZ_LEADER_PREFIX}{leader_state}")?;

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
                        true => ("--abbr ", "commandline edit --insert ' ';", ""),
                        false => ("", "", "-A"),
                    };

                    write!(&mut buffer,
                        "
$env.config.keybindings ++= [
    {{
      name: blz_{0}
      modifier: {2}
      keycode: {3}
      mode: emacs
      event: {{
        send: executehostcommand,
        cmd: \"let tmpfile = (mktemp -p /tmp); blz porcelain leader-key {0} {5} --tmpfile $tmpfile; commandline edit {1} --insert (cat $tmpfile);{4} rm $tmpfile\"
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
