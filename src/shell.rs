use crate::{SHELL, Shell, is_nushell, yml::LeaderKeys};
use anyhow::Result;
use log::debug;

pub mod nu_hook;
pub mod zsh_hook;

pub fn print_leader_state(leaders: &Option<&Vec<LeaderKeys>>) {
    println!("{}", leaders_to_state(leaders));
}

pub fn leaders_to_state(leaders: &Option<&Vec<LeaderKeys>>) -> String {
    leaders.map_or_else(
        || "none".to_string(),
        |l| {
            let mut out = vec![];

            for leader in l {
                out.push(format!(
                    "{}_e{}_a{}",
                    leader.sanitized_name(),
                    leader.exec_mode,
                    leader.abbr_mode,
                ));
            }

            out.sort();
            out.join("|")
        },
    )
}

/// Checks if the leader key state from the env is up to date. Returs an error if not.
pub fn check_leaders(leaders: &Option<&Vec<LeaderKeys>>) -> Result<()> {
    let var = std::env::var("BLZ_LEADER_STATE");
    debug!("Check leader keys: {leaders:?}");
    debug!("BLZ_LEADER_STATE = {var:?}");

    let shell = *SHELL.lock().unwrap();

    let (which, _or) = match shell {
        Shell::Zsh => (".zshrc", ", or run 'source ~/.zshrc'"),
        Shell::Nu => ("nu config", ""),
    };

    match var {
        Ok(it) => {
            if it != leaders_to_state(leaders) {
                // nushell doesn't make it so easy to refresh the environment, so we'll allow an
                // override to quiet this message.
                if is_nushell() && std::env::var("BLZ_STFU").is_ok_and(|it| it != "false") {
                    return Ok(());
                }

                anyhow::bail!(match shell {
                    Shell::Zsh =>
                        "The '.zshrc' needs to be sourced since the leader keys have changed since BLZ was last initialised. \nPlease run 'source ~/.zshrc'.",
                    Shell::Nu =>
                        "The nu session needs to be updated since the BLZ leader keys have changed. This requires opening a new shell with a new environment; sourcing the config and 'exec nu' won't work, but you can open a new terminal tab. \nTip: Use '$env.BLZ_STFU = true' in the current session to quiet this message.",
                })
            } else {
                Ok(())
            }
        }
        Err(_) => {
            anyhow::bail!(
                "The 'BLZ_LEADER_STATE' should have been set in the {which}. Please open a new shell{_or}."
            )
        }
    }
}
