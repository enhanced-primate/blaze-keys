use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(arg_required_else_help = true)]
#[command(about = "Keybind and leader-key manager for blazing fast commands in Zsh.", long_about = None)]
pub struct Args {
    #[clap(
        short = 'g',
        long,
        help = "Edit the global config, creating from template if necessary."
    )]
    pub edit_global_config: bool,

    #[clap(
        short = 'l',
        long,
        help = "Edit a local config, creating from template if necessary."
    )]
    pub edit_local_config: bool,

    #[clap(
        short = 'v',
        long,
        help = "Show the keybinds for the current working directory."
    )]
    pub show_keybinds: bool,

    #[clap(
        short,
        long,
        help = "Print the Zsh bindings (this should be used in your ~/.zshrc)."
    )]
    pub zsh_hook: bool,

    #[cfg(debug_assertions)]
    #[clap(short = 's', long, help = "[development] Swap a config in or out.")]
    pub swap_config: Option<String>,

    #[clap(
        short = 'p',
        long,
        help = "Print a template to stdout. Interactively select if no name is provided."
    )]
    pub print_template: Option<Option<String>>,

    #[clap(subcommand)]
    pub porcelain: Option<PorcelainWrapper>,
}

#[derive(Subcommand, Debug)]
pub enum PorcelainWrapper {
    Porcelain {
        #[clap(subcommand)]
        inner: Porcelain,

        #[clap(short, long, help = "Ignore when the leader-key state does not match.")]
        ignore_leader_state: bool,
    },
}

#[allow(non_camel_case_types)]
#[derive(Subcommand, Debug)]
pub enum Porcelain {
    #[clap(about = "Triggers the TUI with a given leader key.")]
    leader_key {
        leader: String,

        #[clap(long)]
        tmpfile: String,

        #[clap(long)]
        abbr: bool,
    },
    #[clap(about = "Prints the state of the leader keys.")]
    print_leader_state,
    #[clap(about = "Check the state of the leader keys and exit.")]
    check_leader_state,
    #[clap(about = "Generate the nu sources which are used to bind leader-keys to trigger keys.")]
    generate_nu_source,
    #[clap(about = "Print the path to the nu sources file.")]
    print_nu_source_path,
    #[clap(about = "Emit top-level keybinds.")]
    blat,
}

#[macro_export]
macro_rules! porcelain_get {
    ($args:tt, $which:pat => $then:expr) => {
        $args
            .porcelain
            .as_ref()
            .map(|it| match it {
                PorcelainWrapper::Porcelain { inner, .. } => match inner {
                    $which => Some($then),
                    _ => None,
                },
            })
            .flatten()
    };
}

#[macro_export]
macro_rules! porcelain_get_bool {
    ($args:tt, $which:pat) => {
        $args.porcelain.as_ref().is_some_and(|it| match it {
            PorcelainWrapper::Porcelain { inner, .. } => {
                matches!(inner, $which)
            }
        })
    };
}
