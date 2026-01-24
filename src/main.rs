#[cfg(debug_assertions)]
mod dev;

extern crate termion;

use anyhow::{Result, anyhow};
use blaze_keys::CONFIG_DIR;
use blaze_keys::keys::print_bindkey_zsh;
use blaze_keys::yml::{self, GlobalConfig, LocalConfig};
use blaze_keys::{CONFIG_FILE_NAME, keys::print_human_keys, nodes::Node, nu_hook, zsh_hook};
use blaze_keys::{SHELL, Shell, keys};
use clap::Parser;
use colored::Colorize;
use flexi_logger::{FileSpec, LoggerHandle};
use log::{debug, info};
use std::path::{Path, PathBuf};
use std::{
    env,
    fs::File,
    io::{Write, stdin},
    process::{Command, Stdio},
    time::Duration,
};
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

const LOCAL_TEMPLATE: &str = include_str!("../example-configs/templates/local.yml");
const GLOBAL_TEMPLATE: &str = include_str!("../example-configs/templates/global.all.yml");
const GLOBAL_TEMPLATE_SMALL: &str = include_str!("../example-configs/templates/global.small.yml");
const GLOBAL_TEMPLATE_MINIMAL: &str =
    include_str!("../example-configs/templates/global.minimal.yml");

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(arg_required_else_help = true)]
#[command(about = "Keybind and leader-key manager for blazing fast commands in Zsh.", long_about = None)]
struct Args {
    #[clap(
        short = 'g',
        long,
        help = "Edit the global config, creating from template if necessary."
    )]
    edit_global_config: bool,

    #[clap(
        short = 'l',
        long,
        help = "Edit a local config, creating from template if necessary."
    )]
    edit_local_config: bool,

    #[clap(
        short = 'v',
        long,
        help = "Show the keybinds for the current working directory."
    )]
    show_keybinds: bool,

    #[clap(
        short,
        long,
        help = "Print the Zsh bindings (this should be used in your ~/.zshrc)."
    )]
    zsh_hook: bool,

    #[clap(short = 'L', long)]
    porcelain_leader: Option<String>,

    #[clap(long)]
    porcelain_tmp: Option<String>,

    #[clap(long)]
    porcelain_abbr: bool,

    #[clap(
        short = 'B',
        long,
        action,
        help = "Prints the Zsh keybind commands for the current working directory."
    )]
    porcelain_blat: bool,

    #[clap(long)]
    porcelain_ignore_leader_state: bool,

    #[clap(long)]
    porcelain_print_leader_state: bool,

    #[clap(long)]
    porcelain_check_leader_state_then_exit: bool,

    #[cfg(debug_assertions)]
    #[clap(short = 's', long, help = "[development] Swap a config in or out.")]
    swap_config: Option<String>,

    #[clap(
        short = 'p',
        long,
        help = "Print a template to stdout. Interactively select if no name is provided."
    )]
    print_template: Option<Option<String>>,

    #[clap(
        long,
        help = "Generate the source file with leader-key triggers for nushell"
    )]
    porcelain_generate_nu_source: bool,

    #[clap(
        long,
        help = "Print the path to the source file with leader-key triggers for nushell"
    )]
    porcelain_print_nu_source_path: bool,
}

fn parse_global_keybinds<T>(path: T) -> Option<Result<GlobalConfig>>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    if !PathBuf::from(path).exists() {
        info!("Keybinds global file not found: {}", path.display());
        return None;
    }

    if let Ok(c) = std::fs::read_to_string(path) {
        match serde_yml::from_str(&c) {
            Ok(init_conf) => Some(Ok(init_conf)),
            Err(e) => Some(Err(anyhow::Error::from(e))),
        }
    } else {
        Some(Err(anyhow::anyhow!("Failed to read global config file")))
    }
}

/// Parses the keybinds from the '.blz.yml' file.
///
/// Returns None if the file is absent.
fn parse_local_keybinds() -> Option<Result<LocalConfig>> {
    let fname = CONFIG_FILE_NAME;

    let filename = PathBuf::from(fname);

    if !filename.exists() {
        return None;
    }

    info!("Read file: {filename:?}");

    let content = match std::fs::read_to_string(&filename) {
        Ok(c) => c,
        Err(e) => {
            return Some(Err(anyhow!(
                "Failed to read config file={filename:?}: {e:?}"
            )));
        }
    };

    Some(
        serde_yml::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse config from file={filename:?}; {e:?}")),
    )
}

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

    if let Ok(shell) = std::env::var("BLZ_SHELL")
        && shell.starts_with("nu")
    {
        *SHELL.lock().unwrap() = Shell::Nu;
    }

    let hook = std::panic::take_hook();
    // If we panic during the TUI render, the message may be invisible to the user.
    // We write the error to a file instead.
    std::panic::set_hook(Box::new(move |info| {
        let location = info.location().unwrap();
        let message = info.payload().downcast_ref::<&str>();

        let out = if let Some(message) = message {
            &format!("Message: {}", message)
        } else {
            "Panic occurred without a message."
        };

        let mut file = File::create(".panic.blz").unwrap();
        write!(
            file,
            "A panic occurred in blz: \n{out:?}\nlocation: {location:?}"
        )
        .unwrap();

        eprintln!("Panicked! (location={location}) \nmessage={message:?}");
        hook(info);
    }));

    debug!("Executed in {:?}", std::env::current_dir().unwrap());

    let args = Args::parse();

    if args.porcelain_print_nu_source_path {
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
            Some(template_name) => get_template_by_name(&template_name)
                .ok_or_else(|| anyhow!("Invalid template name: {}", template_name))?,
            None => select_template_interactive("A template will be printed to standard output")?,
        };
        println!("{template}");
        return Ok(());
    }

    let config_file = CONFIG_DIR.join(CONFIG_FILE_NAME);

    if args.edit_global_config {
        edit_config_file(CONFIG_DIR.to_str().unwrap(), &config_file)?;
    }
    if args.edit_local_config {
        let path = PathBuf::from(CONFIG_FILE_NAME);
        if !path.exists() {
            std::fs::write(&path, LOCAL_TEMPLATE)?;
        }
        println!("Created {path:?}.");
        println!(
            "{}{}",
            "ATTENTION".on_cyan(),
            ": You will need to run 'cd .' to refresh the local keybinds.".bright_red()
        );
        edit_config_file(".", &path)?;
    }
    let global_binds = parse_global_keybinds(&config_file).transpose()?;

    if args.zsh_hook {
        zsh_hook::print_zsh_hook(&global_binds);
        return Ok(());
    }
    if args.porcelain_generate_nu_source {
        nu_hook::generate_nu_source(&global_binds)?;
        return Ok(());
    }
    let local_binds = parse_local_keybinds();

    debug!("Global keybinds: {global_binds:?}");
    debug!("Loaded local keybinds: {local_binds:?}");

    let ld = match &global_binds {
        Some(i) => match i.global {
            Some(ref i) => &i.leader_keys.as_ref(),
            _ => &None,
        },
        None => &None,
    };

    if args.porcelain_print_leader_state {
        zsh_hook::print_leader_state(ld);
        return Ok(());
    }
    if !args.porcelain_ignore_leader_state && args.porcelain_leader.is_none()
    /* No need to check when the leader key is set, because that means this leader key is up to date at least */
    {
        zsh_hook::check_leaders(ld)?;

        if args.porcelain_check_leader_state_then_exit {
            return Ok(());
        }
    }

    if let Some(leader) = args.porcelain_leader {
        let leader_keys = Node::root(&global_binds, leader)?;

        let tmp = match &args.porcelain_tmp {
            Some(t) => t,
            None => {
                anyhow::bail!(
                    "'--porcelain-tmp' must be set to the path to a temporary file which is used to store the output"
                )
            }
        };

        leader_keys_tui(leader_keys, args.porcelain_abbr, tmp);
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

fn get_template_by_name(name: &str) -> Option<&'static str> {
    match name {
        "all" | "a" => Some(GLOBAL_TEMPLATE),
        "small" | "s" => Some(GLOBAL_TEMPLATE_SMALL),
        "minimal" | "m" => Some(GLOBAL_TEMPLATE_MINIMAL),
        _ => None,
    }
}

const TEMPLATE_OPTIONS: &str = "The following are available:
[all]     --  Contains a large number of aliases which you can trim down and modify as you like (includes Git, Docker, Cargo etc).
[small]   --  Contains a small number of aliases (mostly for Git). 
[minimal] --  Contains a few commented-out examples, but no aliases by default. 
Which would you like? [all (a), small (s), minimal (m)] --> ";

fn select_template_interactive(intro: &str) -> Result<&'static str> {
    let stdin = std::io::stdin();

    let template = loop {
        let mut line = String::new();

        print!("{intro}. {TEMPLATE_OPTIONS}");

        std::io::stdout().flush()?;
        stdin.read_line(&mut line)?;

        if let Some(template) = get_template_by_name(line.as_str().trim_end()) {
            break template;
        } else {
            println!("Invalid input.");
            continue;
        }
    };

    Ok(template)
}

fn create_global_config_interactive(config_file: &PathBuf) -> Result<()> {
    println!(
        "{}: You can view the templates first by using Ctrl+C and then running 'blz --print-template'.",
        "TIP".on_bright_cyan().bright_white()
    );
    let template =
        select_template_interactive("The global config will be created from a template")?;

    std::fs::write(config_file, template)?;
    println!("Created {config_file:?}.");

    Ok(())
}

fn edit_config_file(config_dir: &str, config_file: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(config_dir)?;

    if !config_file.exists() {
        create_global_config_interactive(config_file)?;
        std::thread::sleep(Duration::from_millis(750));
    }

    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            let editor = find_program(&["nvim", "code", "emacs", "zed", "vim", "nano", "vi"])
                .expect("Failed to find an editor! Export the 'EDITOR' env variable.");

            println!(
                "Looked for an editor and chose {editor:?}. You can override by exporting the 'EDITOR' env variable."
            );
            editor
        });

    Command::new(editor)
        .arg(config_file)
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()?
        .wait()?;

    std::process::exit(0);
}

/// Used to find the optimal editor.
fn find_program(programs: &[&str]) -> Option<String> {
    for program in programs {
        let output = Command::new("which").arg(program).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    return Some(program.to_string());
                }
            }
            Err(_) => continue,
        }
    }

    None
}
