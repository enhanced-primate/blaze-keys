use clap::Parser;
use colored::Colorize;
use std::env;
use std::fs;
use std::io;
use std::process;

// Define a struct to hold the tutorial phases
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Next,
    Reset,
    Repeat,
    Previous,
}

impl Cli {
    fn run(&self) {
        match self.command {
            Commands::Next => {
                run_tutorial_phase(get_and_update_phase(1));
            }

            Commands::Repeat => {
                run_tutorial_phase(get_and_update_phase(0));
            }
            Commands::Previous => {
                run_tutorial_phase(get_and_update_phase(-1));
            }
            Commands::Reset => {
                save_progress(0).expect("Failed to reset progress");
                run_tutorial_phase(get_and_update_phase(0));
            }
        }
    }
}

struct TutorialPhase {
    instruction: String,
}

impl TutorialPhase {
    fn new<T, R>(prompt: T, tip: R) -> TutorialPhase
    where
        T: AsRef<str>,
        R: AsRef<str>,
    {
        TutorialPhase {
            instruction: instruction_str(prompt.as_ref(), tip.as_ref()),
        }
    }
}

fn get_progress_file() -> String {
    format!(
        "/tmp/tutorial_progress_{}",
        env::var("USER").unwrap_or("unknown".to_string())
    )
}

fn get_and_update_phase(delta: i32) -> u32 {
    let progress_file = get_progress_file();

    let current = match fs::read_to_string(progress_file) {
        Ok(contents) => contents.trim().parse().unwrap_or_default(),
        Err(_) => -delta,
    };

    let new_phase = (current + delta).max(0) as u32;

    save_progress(new_phase).expect("Failed to write to file storing the tutuorial phase!");
    new_phase
}

fn save_progress(phase: u32) -> io::Result<()> {
    let progress_file = get_progress_file();
    fs::write(progress_file, phase.to_string())
}

fn instruction_str(prompt: &str, tip: &str) -> String {
    format!(
        "{}\n{} {}: {}",
        prompt.bright_green(),
        ">".bright_cyan(),
        "Tip".bright_blue(),
        tip.yellow()
    )
}

fn run_tutorial_phase(phase: u32) {
    let phases = [
        TutorialPhase::new(
            format!(
                "Welcome to the tutorial. Try running '{}' using blaze-keys. Then run '{}' to progress.",
                "git status".bright_cyan(),
                "tut next".bright_red(),
            ),
            format!(
                "Invoke '{}' mode with 'Ctrl+s', then type 'gs'.",
                "exec".bright_cyan()
            ),
        ),
        TutorialPhase::new(
            format!(
                "Now try running a command which takes arguments; you'll want to use '{}' mode. Try 'git checkout -b test' using blaze-keys, then run 'tut next'.",
                "abbr".bright_cyan()
            ),
            format!(
                "Invoke '{}' mode with 'Alt+s', then type 'gcb'. Then you can type your branch name and press enter.",
                "abbr".bright_cyan()
            ),
        ),
        TutorialPhase::new(
            format!(
                "Some keybinds change as you 'cd', based on {}. This sample config defines 'Alt+b' to build. Try it in {} and then in '{}', then run 'tut next'.",
                "profiles".bright_cyan(),
                "this directory".bright_cyan(),
                "python_sample".bright_cyan(),
            ),
            "Press 'Alt+b' to run a make build. Then run 'cd python_sample' and press 'Alt+b' again to run a 'uv' build.",
        ),
        TutorialPhase::new(
            format!(
                "You can also bind to zsh builtins. Try '{}', which is useful when typing a command and needing to run another.",
                "push-line".bright_cyan()
            ),
            format!(
                "Type '{0}', but don't press enter; you forgot to create the dir. Use '{1}' to push-line, run '{2}' and then '{0}' is restored.",
                "touch foo/bar".bright_cyan(),
                "Alt+p".bright_cyan(),
                "mkdir foo".bright_cyan(),
            ),
        ),
    ];

    if phase < phases.len() as u32 {
        println!("{}", phases[phase as usize].instruction);
        process::exit(0);
    } else {
        println!(
            "{}",
            &format!(
                "Tutorial completed! Use '{}' if you'd like to start again, or '{}' to go back.",
                "tut reset".bright_cyan(),
                "tut previous".bright_cyan()
            )
            .bright_green()
        );
        process::exit(0);
    }
}

fn main() {
    let cli = Cli::parse();
    cli.run();
}
