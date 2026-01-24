use crate::{
    keys::{self, NuKey},
    yml::GlobalConfig,
};
use anyhow::Result;

pub fn print_nu_hook(global: &Option<GlobalConfig>) -> Result<()> {
    if let Some(g) = global.as_ref().and_then(|g| g.global.as_ref()) {
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
                        true => ("--porcelain-abbr ", "commandline edit --insert ' ';", ""),
                        false => ("", "", "-A"),
                    };

                    println!(
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
                    )
                }
            }
        }
        println!("##### blaze-keys: end");
    }

    Ok(())
}
