mod cli;
mod configs;
mod panic;

#[cfg(debug_assertions)]
mod dev;

extern crate termion;

use anyhow::{Result, anyhow};
use blaze_keys::keys::print_bindkey_zsh;
use blaze_keys::yml::{self};
use blaze_keys::{CONFIG_DIR, shell};
use blaze_keys::{
    CONFIG_FILE_NAME, keys::print_human_keys, nodes::Node, shell::nu_hook, shell::zsh_hook,
};
use blaze_keys::{SHELL, Shell, keys};
use clap::Parser;
use colored::Colorize;
use flexi_logger::{FileSpec, LoggerHandle};
use log::debug;
use std::io::stdin;
use std::path::PathBuf;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

use crate::cli::{Args, Porcelain, PorcelainWrapper};

fn setup_logging() -> Option<LoggerHandle> {
    if let Ok(blz_log) = std::env::var("BLZ_LOG") {
        let file_spec = FileSpec::try_from("/tmp/blz.log").unwrap();

        let logger = flexi_logger::Logger::try_with_str(&blz_log)
            .unwrap()
            .log_to_file(file_spec)
            .write_mode(flexi_logger::WriteMode::BufferAndFlush)
            .append()
            .start()
            .unwrap();

        log::warn!("Set up logging");
        return Some(logger);
    }

    None
}

/// Panic if running as root and BLZ_ALLOW_ROOT is not experted.
fn check_root() {
    let is_root = unsafe { libc::geteuid() == 0 };

    if is_root {
        if let Ok(value) = std::env::var("BLZ_ALLOW_ROOT")
            && value != "0"
        {
            return;
        }
        panic!(
            "ERROR: blz is not allowed to trigger as root unless 'BLZ_ALLOW_ROOT=1' is exported"
        );
    }
}

fn leader_keys_tui(leader_keys: Node, abbr: bool, tmp: &str) {
    let stdin = stdin();

    let tty = termion::get_tty().unwrap();

    let term = tty
        .into_raw_mode()
        .unwrap()
        .into_alternate_screen()
        .unwrap();

    let tui = blaze_keys::tui::Tui::new(term, tmp.to_string(), &leader_keys, abbr);

    tui.run(stdin);
}

fn main() -> Result<(), anyhow::Error> {
    let _logger = setup_logging();
    check_root();

    panic::register_hook();

    if let Ok(shell) = std::env::var("BLZ_SHELL")
        && shell.starts_with("nu")
    {
        *SHELL.lock().unwrap() = Shell::Nu;
    }

    debug!("Executed in {:?}", std::env::current_dir().unwrap());

    let args = Args::parse();

    if porcelain_get_bool!(args, Porcelain::print_nu_source_path) {
        println!("{}", nu_hook::nu_source_location());
        return Ok(());
    }

    #[cfg(debug_assertions)]
    if let Some(name) = args.swap_config {
        dev::swap_config(&name, CONFIG_DIR.to_str().unwrap())?;
        return Ok(());
    }

    if let Some(template_option) = args.print_template {
        let template = match template_option {
            Some(template_name) => configs::get_template_by_name(&template_name)
                .ok_or_else(|| anyhow!("Invalid template name: {}", template_name))?,
            None => configs::select_template_interactive(
                "A template will be printed to standard output",
            )?,
        };
        println!("{template}");
        return Ok(());
    }

    let config_file = CONFIG_DIR.join(CONFIG_FILE_NAME);

    if args.edit_global_config {
        configs::edit_config_file(CONFIG_DIR.to_str().unwrap(), &config_file)?;
    }
    if args.edit_local_config {
        let path = PathBuf::from(CONFIG_FILE_NAME);
        if !path.exists() {
            std::fs::write(&path, configs::LOCAL_TEMPLATE)?;
        }
        println!("Created {path:?}.");
        println!(
            "{}{}",
            "ATTENTION".on_cyan(),
            ": You will need to run 'cd .' to refresh the local keybinds.".bright_red()
        );
        configs::edit_config_file(".", &path)?;
    }
    let global_binds = configs::parse_global_keybinds(&config_file).transpose()?;

    if args.zsh_hook {
        zsh_hook::print_zsh_hook(&global_binds);
        return Ok(());
    }
    if porcelain_get_bool!(args, Porcelain::generate_nu_source) {
        nu_hook::generate_nu_source(&global_binds)?;
        return Ok(());
    }
    let local_binds = configs::parse_local_keybinds();

    debug!("Global keybinds: {global_binds:?}");
    debug!("Loaded local keybinds: {local_binds:?}");

    let ld = match &global_binds {
        Some(i) => match i.global {
            Some(ref i) => &i.leader_keys.as_ref(),
            _ => &None,
        },
        None => &None,
    };

    if porcelain_get_bool!(args, Porcelain::print_leader_state) {
        shell::print_leader_state(ld);
        return Ok(());
    }

    if !args.porcelain.as_ref().is_some_and(|it| match it {
        PorcelainWrapper::Porcelain {
            ignore_leader_state,
            ..
        } => *ignore_leader_state,
    }) && !porcelain_get_bool!(
        args,
        Porcelain::leader_key {
            leader: _,
            tmpfile: _,
            abbr: _
        }
    )
    /* No need to check when the leader key is set, because that means this leader key is up to date at least */
    {
        shell::check_leaders(ld)?;

        if porcelain_get_bool!(args, Porcelain::check_leader_state) {
            return Ok(());
        }
    }

    if let Some((leader, tmpfile, abbr)) = porcelain_get!(args, Porcelain::leader_key {leader, tmpfile, abbr} => (leader, tmpfile, abbr))
    {
        let leader_keys = Node::root(&global_binds, leader.to_owned())?;

        leader_keys_tui(leader_keys, *abbr, tmpfile);
        return Ok(());
    }

    let emitter = if args.show_keybinds {
        print_human_keys
    } else {
        print_bindkey_zsh
    };

    if let Some(ref global_binds) = global_binds {
        global_binds.emit(&emitter)?;
    }

    if let Some(binds) = local_binds {
        let binds = binds?;

        if let Some(inherits_profiles) = binds.inherits {
            let profiles = global_binds
                .ok_or_else(|| anyhow!("Error: can only use 'inherits' in local config if profiles are defined in global.blz.yml, but the latter seems to be absent"))?
                .profiles
                .ok_or_else(|| anyhow!("Error: can only use 'inherits' in local config if profiles are defined in global.blz.yml, but profiles seem to be absent in the latter"))?;

            let profiles: fnv::FnvHashMap<&str, &yml::Profile> = profiles
                .iter()
                .map(|profile| (profile.name.as_str(), profile))
                .collect();

            for p in inherits_profiles {
                let prof = profiles.get(p.as_str()).ok_or_else(|| anyhow!("Error: Local config inherits profile {p:?} which does not exist in global config"))?;

                debug!("Inherit profile {p:?}");
                if let Some(ref kb) = prof.keybinds {
                    keys::emit_keybinds(kb, &emitter)?;
                }
            }
        }

        if let Some(keybinds) = binds.keybinds {
            debug!("Emit keybinds from local config");
            keys::emit_keybinds(&keybinds, &emitter)?;
        }
    } else {
        debug!("No local keybinds found");
    }

    Ok(())
}
