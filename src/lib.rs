use std::{path::PathBuf, sync::Mutex};

use once_cell::sync::Lazy;

pub mod keys;
pub mod nodes;
pub mod shell;
pub mod tui;
pub mod yml;

pub const CONFIG_FILE_NAME: &str = ".blz.yml";
pub const NU_SOURCE_NAME: &str = ".leader_keys.nu";

pub static CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    shellexpand::tilde("~/.config/blaze-keys")
        .to_string()
        .into()
});

#[derive(PartialEq, Copy, Clone)]
pub enum Shell {
    Zsh,
    Nu,
}

pub static SHELL: Mutex<Shell> = Mutex::new(Shell::Zsh);

pub fn is_nushell() -> bool {
    *SHELL.lock().unwrap() == Shell::Nu
}
