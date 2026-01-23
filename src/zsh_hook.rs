use anyhow::Result;
use log::debug;

use crate::{
    keys::{self, KeyOrLeader},
    yml::{GlobalConfig, LeaderKeys},
};

fn print_zsh_chpwd_hook() {
    println!(
        "
## blaze-keys: start
##
## Warning: You should avoid putting this output directly in your .zshrc in case the content changes in future versions.
## This is why the command is called dynamically to initialise the bindings in the .zshrc.
##

##### The zsh hook which is called on changing directories. #####
autoload -U add-zsh-hook

run_on_cd() {{
    source <(blz --porcelain-blat)
}}

add-zsh-hook chpwd run_on_cd
source <(blz --porcelain-ignore-leader-state)
"
    );
}

pub fn leaders_to_state(leaders: &Option<&Vec<LeaderKeys>>) -> String {
    leaders.map_or_else(
        || "none".to_string(),
        |l| {
            let mut out = vec![];

            for leader in l {
                out.push(leader.sanitized_name());
                out.push(leader.exec_mode.clone());
                out.push(leader.abbr_mode.clone());
            }

            out.sort();
            out.join("|")
        },
    )
}

pub fn check_leaders(leaders: &Option<&Vec<LeaderKeys>>) -> Result<()> {
    let var = std::env::var("BLZ_LEADER_STATE");
    debug!("Check leader keys: {leaders:?}");
    debug!("BLZ_LEADER_STATE = {var:?}");

    match var {
        Ok(it) => {
            if it != leaders_to_state(leaders) {
                anyhow::bail!(
                    "\nThe .zshrc needs to be sourced since the leader keys have changed since BLZ was last initialised. \nPlease run 'source ~/.zshrc'."
                )
            } else {
                Ok(())
            }
        }
        Err(_) => {
            anyhow::bail!(
                "The 'BLZ_LEADER_STATE' should have been set in the .zshrc. Please run `source ~/.zshrc`."
            )
        }
    }
}

pub fn print_export_leaders(leaders: &Option<&Vec<LeaderKeys>>) {
    println!("export BLZ_LEADER_STATE='{}'", leaders_to_state(leaders));
}

/// Prints the code required to integrate the program with Zsh.
pub fn print_zsh_hook(global: &Option<GlobalConfig>) {
    print_zsh_chpwd_hook();

    if let Some(g) = global.as_ref().and_then(|g| g.global.as_ref()) {
        print_export_leaders(&g.leader_keys.as_ref());

        if let Some(ref leaders) = g.leader_keys {
            if !leaders.is_empty() {
                println!("##### The zsh widgets which provide the leader key functionality. #####");
            }
            for (index, leader) in leaders.iter().enumerate() {
                for (_i, k) in [&leader.exec_mode, &leader.abbr_mode].iter().enumerate() {
                    let abbr = _i == 1;
                    let key = keys::get_key_zsh_representation(k);

                    let func_name = format!(
                        "_zsh_leader{index}{}",
                        if abbr { "_abbr " } else { "_exec" }
                    );

                    let key_zsh = match &key {
                        Some(KeyOrLeader::Key(k)) => k,
                        _ => panic!("invalid keybind for leader"),
                    };
                    let (flag, spacing, zle_accept) = match abbr {
                        true => ("--porcelain-abbr ", " ", ""),
                        false => ("", "", "\n    zle accept-line"),
                    };

                    println!(
                        "function {} {{
  tmpfile=$(mktemp)
  blz --porcelain-leader {} {}--porcelain-tmp $tmpfile < /dev/tty
  content=$(cat $tmpfile)

  if [[ $content =~ '^zle .*' ]]; then
    eval $content
  else
    LBUFFER+=\"$(cat $tmpfile){}\"{}
  fi

  rm $tmpfile
}}

zle -N {func_name}
bindkey '{key_zsh}' {func_name}
",
                        func_name,
                        leader.sanitized_name(),
                        flag,
                        spacing,
                        zle_accept,
                    )
                }
            }
        }
        println!("## blaze-keys: end");
    } else {
        print_export_leaders(&None);
    }
}
