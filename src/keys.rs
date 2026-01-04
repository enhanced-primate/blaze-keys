#![allow(dead_code)]

use crate::yml::Keybind;
use log::info;
use once_cell::sync::Lazy;
use phf::phf_map;
use regex::Regex;

static FUNCTION_KEYS: phf::Map<&str, &str> = phf_map! {
    "F1" => "^[OP",
    "F2" => "^[OQ",
    "F3" => "^[OR",
    "F4" => "^[OS",
    "F5" => "^[[15~",
    "F6" => "^[[17~",
    "F7" => "^[[18~",
    "F8" => "^[[19~",
    "F9" => "^[[20~",
    "F10" => "^[[21~",
    "F11" => "^[[23~",
    "F12" => "^[[24~",
};

static REGEX_ALT: Lazy<Regex> = Lazy::new(|| Regex::new("^(Alt|A|alt)-").unwrap());
static REGEX_CTRL: Lazy<Regex> = Lazy::new(|| Regex::new("^(Ctrl|C|ctrl)-").unwrap());
static REGEX_LEADER_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*--").unwrap());
static REGEX_LEADER_COMBO: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*([^=]*?)\s*=\s*([^=]*)$").unwrap());

static REGEX_COMBO_VALIDATE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(<.*?>)(?:|[^>^<])").unwrap());

pub fn get_key_name(key: &str) -> Option<&str> {
    FUNCTION_KEYS.get(key).map(|s| s.as_ref())
}

#[allow(dead_code)]
pub enum KeyOrLeader {
    Key(String),
    LeaderCombo(String),
}

fn alt_starter_key() -> &'static str {
    "\\e"
}

pub fn get_key_zsh_representation(key: &str) -> Option<KeyOrLeader> {
    let starter;
    let leftover;

    if let Some(f_alt) = REGEX_ALT.find(key) {
        let pos = f_alt.end();
        leftover = key.split_at(pos).1;
        starter = alt_starter_key();
    } else if let Some(f_ctrl) = REGEX_CTRL.find(key) {
        let pos = f_ctrl.end();
        leftover = key.split_at(pos).1;
        starter = "^";
    } else {
        match FUNCTION_KEYS.get(key) {
            Some(func_key) => return Some(KeyOrLeader::Key(func_key.to_string())),
            _ => return None,
        }
    }

    Some(KeyOrLeader::Key(format!("{starter}{leftover}")))
}

pub fn print_human_keys(
    keybind: &Keybind,
    key_raw: &str,
    command: &Option<String>,
    zle: &Option<String>,
) -> anyhow::Result<()> {
    let out;

    if let Some(command) = command {
        out = format!("'{command}'");
    } else if let Some(zle) = zle {
        out = format!("'{}' (zle builtin)", zle);
    } else {
        anyhow::bail!("The config item for '{key_raw}' must set either 'command' or 'zle'.")
    }

    let width = 7;
    println!("{:<width$} ----->  {out}", keybind.key);

    Ok(())
}

pub fn print_bindkey_zsh(
    keybind: &Keybind,
    key_raw: &str,
    command: &Option<String>,
    zle: &Option<String>,
) -> anyhow::Result<()> {
    let command_out;

    if let Some(command) = command {
        command_out = format!("bindkey -s '{}' \"{}^M\"", key_raw, command);
    } else if let Some(zle) = zle {
        command_out = format!("bindkey '{}' \"{}\"", key_raw, zle);
    } else {
        anyhow::bail!("The config item for '{key_raw}' must set either 'command' or 'zle'.");
    }

    let width = 50;
    let command_out = format!("{:<width$}  # <--- {}", command_out, keybind.key);

    println!("{command_out}");
    info!("{command_out}");

    Ok(())
}

pub fn emit_keybinds<T>(keybinds: &[Keybind], print_bindkey_fn: &T) -> anyhow::Result<()>
where
    T: Fn(&Keybind, &str, &Option<String>, &Option<String>) -> anyhow::Result<()>,
{
    for k in keybinds {
        let Keybind {
            key,
            command,
            zle,
            raw,
        } = k;

        let repr = get_key_zsh_representation(&key);

        if raw.unwrap_or(false) {
            print_bindkey_fn(k, &key, command, zle)?;
        } else if let Some(repr) = repr {
            match repr {
                KeyOrLeader::Key(repr) => {
                    print_bindkey_fn(k, &repr, command, zle)?;
                }
                KeyOrLeader::LeaderCombo(_) => (), // We don't need to handle this here.
            }
        } else {
            anyhow::bail!(
                "Unable to generate keybind '{key}', key is not supported - if this is not a typo, please use 'raw: true' in the .yml file for this key"
            );
        }
    }
    Ok(())
}

#[test]
fn test_parse_combo() {
    assert!(parse_combo("<c-s>glo = git log --oneline").is_err());
    assert!(parse_combo("<C-s>g<a-g>lo = git log --oneline").is_err());
    assert!(parse_combo("test test").is_err());

    parse_combo("<C-s>glo = git log --oneline").unwrap();
    parse_combo("<C-s>g<A-g>lo = git log --oneline").unwrap();
}

pub(crate) fn parse_combo(line: &str) -> Result<Option<(String, String)>, anyhow::Error> {
    if REGEX_LEADER_COMMENT.is_match(line) || line.is_empty() {
        return Ok(None);
    }

    if let Some(captures) = REGEX_LEADER_COMBO.captures(line) {
        if captures.len() != 3 {
            return Err(anyhow::anyhow!(
                "Bad format of string in leader combo: {line:?}"
            ));
        }

        let combo = captures.get(1).unwrap().as_str().to_string();
        let command = captures.get(2).unwrap().as_str().to_string();

        for c in REGEX_COMBO_VALIDATE.captures_iter(&combo) {
            for _match in c.iter().flatten() {
                let s = _match.as_str();
                let s = &s[1..s.len() - 1];

                if !REGEX_CTRL.is_match(s) && !REGEX_ALT.is_match(s) {
                    anyhow::bail!(
                        "Bad format of angle brackets in leader combo: {combo:?} (s={s:?})"
                    );
                }
            }
        }

        Ok(Some((combo, command)))
    } else {
        Err(anyhow::anyhow!(
            "Bad format of string in leader combo: {line:?}"
        ))
    }
}
