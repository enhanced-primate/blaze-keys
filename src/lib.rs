use std::sync::Mutex;

pub mod keys;
pub mod nodes;
pub mod nu_hook;
pub mod tui;
pub mod yml;
pub mod zsh_hook;

pub const CONFIG_FILE_NAME: &str = ".blz.yml";

#[derive(PartialEq)]
pub enum Shell {
    Zsh,
    Nu,
}

pub static SHELL: Mutex<Shell> = Mutex::new(Shell::Zsh);

pub fn is_nushell() -> bool {
    *SHELL.lock().unwrap() == Shell::Nu
}
