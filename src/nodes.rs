use log::debug;
use once_cell::sync::Lazy;
use regex::Regex;
use std::ops::Deref;

use crate::{
    keys::{self},
    yml::GlobalConfig,
};

static CTRL_ALT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(<(?:Ctrl|Alt|C|A)-.>|.)").unwrap());

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub children: fnv::FnvHashMap<CharWithModifiers, Node>,
    pub command: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum CharWithModifiers {
    Ctrl(char),
    Alt(char),
    Unmodified(char),
}

impl Deref for CharWithModifiers {
    type Target = char;

    fn deref(&self) -> &Self::Target {
        self.bare()
    }
}

impl CharWithModifiers {
    pub fn bare(&self) -> &char {
        match self {
            Self::Ctrl(c) | Self::Alt(c) | Self::Unmodified(c) => c,
        }
    }
    pub fn str_short(&self) -> String {
        match self {
            Self::Ctrl(c) => format!("<C-{c}>"),
            Self::Alt(c) => format!("<A-{c}>"),
            Self::Unmodified(c) => format!("{c}"),
        }
    }
}

impl From<char> for CharWithModifiers {
    fn from(c: char) -> CharWithModifiers {
        CharWithModifiers::Unmodified(c)
    }
}

impl Node {
    pub fn root(global: &Option<GlobalConfig>, leader_chosen: String) -> anyhow::Result<Self> {
        let mut root = Node::default();

        if let Some(global) = global
            && let Some(ref conf) = global.global
            && let Some(ref leaderkeys) = conf.leader_keys
        {
            for _leader in leaderkeys {
                if _leader.sanitized_name() == leader_chosen {
                    for line in _leader.combos.lines() {
                        match keys::parse_combo(line) {
                            Ok(Some((combo, command))) => {
                                debug!("Found leader combo: {combo} -> {command}");

                                let mut chars: Vec<CharWithModifiers> = vec![];

                                for cap in CTRL_ALT_REGEX.captures_iter(&combo) {
                                    match cap.get(1) {
                                        Some(m) => {
                                            let key = m.as_str();

                                            if key.starts_with("<C") {
                                                let c = key.chars().nth_back(1).unwrap();
                                                chars.push(CharWithModifiers::Ctrl(c));
                                            } else if key.starts_with("<A") {
                                                let c = key.chars().nth_back(1).unwrap();
                                                chars.push(CharWithModifiers::Alt(c));
                                            } else {
                                                chars.push(CharWithModifiers::Unmodified(
                                                    key.chars().nth(0).unwrap(),
                                                ));
                                            }
                                        }
                                        _ => anyhow::bail!("Invalid syntax in combo: {combo}"),
                                    };
                                }
                                debug!("new chars: {chars:?}");

                                let mut node = root.find_node(*chars.first().unwrap());

                                for char in chars[1..chars.len()].iter() {
                                    node = node.find_node(*char);
                                }

                                if node.command.is_some() {
                                    anyhow::bail!("combo {combo:?} is defined multiple times");
                                }
                                node.command = Some(command);
                            }
                            Ok(None) => continue,
                            Err(e) => return Err(e),
                        }
                    }
                    break;
                }
            }
        }

        Ok(root)
    }

    fn find_node(&mut self, char: CharWithModifiers) -> &mut Node {
        let ch = &mut self.children;
        let node = Node::default();

        ch.entry(char).or_insert(node)
    }
}
