use crate::{
    keys::{self, KeyOrLeader, NuKey},
    yml::{GlobalConfig, LeaderKeys},
};
use anyhow::Result;

use crate::zsh_hook;

pub fn print_nu_hook(global: &Option<GlobalConfig>) -> Result<()> {
    if let Some(g) = global.as_ref().and_then(|g| g.global.as_ref()) {
        zsh_hook::print_export_leaders(&g.leader_keys.as_ref());

        if let Some(ref leaders) = g.leader_keys {
            if !leaders.is_empty() {
                println!(
                    "##### blaze-keys: start\n##### The nu widgets which provide the leader key functionality."
                );
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
                        true => ("--porcelain-abbr ", " ", ""),
                        false => ("", "", "-A"),
                    };

                    println!(
                        "
$env.config.keybindings ++= [
    {{
      name: blz_{1}
      modifier: {3}
      keycode: {4}
      mode: emacs
      event: {{
        send: executehostcommand,
        cmd: \"blz --porcelain-leader {1} --porcelain-tmp {0}; commandline edit {2} --insert (cat {0})\"
      }} }}
]
",
                        flag,
                        leader.sanitized_name(),
                        accept_flag,
                        modifier,
                        char,
                    )
                }
            }
        }
        println!("##### blaze-keys: end");
    } else {
        zsh_hook::print_export_leaders(&None);
    }

    Ok(())
}
