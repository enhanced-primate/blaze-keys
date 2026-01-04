use std::path::{Path, PathBuf};

use log::{debug, info};
use serde::*;

use crate::keys::{self, emit_keybinds};

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub global: Option<Global>,
    pub profiles: Option<Vec<Profile>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalConfig {
    pub inherits: Option<Vec<String>>,
    pub keybinds: Option<Vec<Keybind>>,
}

impl GlobalConfig {
    pub fn emit<T>(&self, emitter: &T) -> anyhow::Result<()>
    where
        T: Fn(&Keybind, &str, &Option<String>, &Option<String>) -> anyhow::Result<()>,
    {
        info!("Emit global keybinds");

        if let Some(ref global) = self.global {
            keys::emit_keybinds(&global.keybinds, emitter)?;
        }

        if let Some(ref profiles) = self.profiles {
            for profile in profiles {
                if let Some(ref keybinds) = profile.keybinds
                    && profile.evaluate_conditions()
                {
                    info!("Apply profile: {}", profile.name);
                    emit_keybinds(keybinds, emitter)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub keybinds: Option<Vec<Keybind>>,
    pub conditions: Option<Vec<Condition>>,
}

impl Profile {
    fn evaluate_conditions(&self) -> bool {
        if let Some(ref conditions) = self.conditions {
            for c in conditions {
                match c {
                    Condition::Glob { glob } => {
                        if is_glob_found(glob) {
                            debug!("Matched glob '{glob}' in condition");
                            return true;
                        }
                    }
                    Condition::Within { within } => {
                        let cwd: String = std::env::current_dir()
                            .expect("Failed to get current directory")
                            .into_os_string()
                            .to_str()
                            .expect("Failed to convert string to str")
                            .to_string();

                        if is_within_dir(&cwd, within) {
                            debug!("Matched within '{within}' in condition");
                            return true;
                        }
                    }
                }
            }
        }

        false
    }
}

#[test]
fn test_within_dir() {
    assert!(is_within_dir("/home/user/test", "/home/user"));
    assert!(!is_within_dir("/home/user/", "/home/user/test"));
    assert!(is_within_dir("/home/user/", "/home/user"));
    assert!(is_within_dir("/home/user", "/home/user/"));
    assert!(is_within_dir("/home/user/test/2", "/home/user/test"));

    assert!(!is_within_dir("/home/user/test2/2", "/home/user/test"));
    assert!(is_within_dir("/home/user/test2/2", "/"));

    assert!(is_within_dir("~/test2/2", "~"));
}

fn normalize_path(path: &Path) -> Option<PathBuf> {
    let mut test_path = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                if !test_path.pop() {
                    return None;
                }
            }
            std::path::Component::CurDir => {}
            _ => test_path.push(component.as_os_str()),
        }
    }
    Some(test_path)
}

fn is_within_dir(dir: &str, within: &str) -> bool {
    let expanded_dir = shellexpand::tilde(&dir);
    let expanded_within = shellexpand::tilde(&within);

    if let (Some(norm_path), Some(norm_base)) = (
        normalize_path(&PathBuf::from(&expanded_dir.as_ref())),
        normalize_path(&PathBuf::from(expanded_within.as_ref())),
    ) {
        norm_path.starts_with(norm_base)
    } else {
        false
    }
}

fn is_glob_found(pattern: &str) -> bool {
    glob::glob(pattern)
        .expect("could not parse glob pattern")
        .next()
        .is_some()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    Glob { glob: String },
    Within { within: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Global {
    pub keybinds: Vec<Keybind>,
    pub leader_keys: Option<Vec<LeaderKeys>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderKeys {
    name: String,
    pub exec_mode: String,
    pub abbr_mode: String,
    pub combos: String,
}

impl LeaderKeys {
    pub fn sanitized_name(&self) -> String {
        self.name.replace(" ", "_")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybind {
    pub key: String,
    pub command: Option<String>,
    pub zle: Option<String>,
    pub raw: Option<bool>,
}
