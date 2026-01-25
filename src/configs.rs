use anyhow::Result;
use anyhow::anyhow;
use blaze_keys::{
    CONFIG_FILE_NAME,
    yml::{GlobalConfig, LocalConfig},
};
use colored::Colorize;
use log::info;
use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

pub const LOCAL_TEMPLATE: &str = include_str!("../example-configs/templates/local.yml");
const GLOBAL_TEMPLATE: &str = include_str!("../example-configs/templates/global.all.yml");
const GLOBAL_TEMPLATE_SMALL: &str = include_str!("../example-configs/templates/global.small.yml");
const GLOBAL_TEMPLATE_MINIMAL: &str =
    include_str!("../example-configs/templates/global.minimal.yml");

pub fn get_template_by_name(name: &str) -> Option<&'static str> {
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

pub fn select_template_interactive(intro: &str) -> Result<&'static str> {
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

pub fn parse_global_keybinds<T>(path: T) -> Option<Result<GlobalConfig>>
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
pub fn parse_local_keybinds() -> Option<Result<LocalConfig>> {
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

pub fn create_global_config_interactive(config_file: &PathBuf) -> Result<()> {
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

pub fn edit_config_file(config_dir: &str, config_file: &PathBuf) -> Result<()> {
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
