extern crate termion;

use colored::{Color, Colorize};
use itertools::Itertools;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;
use termion::event::Key;
use termion::input::TermRead;

use crate::nodes::{CharWithModifiers, CharWithModifiersAndValidity, Node};

trait TruncateWithEllipsis {
    fn ellipsis(&self, start: u16, max_len: u16) -> String;
}

const ELLIPSIS: &str = "...";
const ELLIPSIS_LEN: usize = ELLIPSIS.len();
const FINAL: &str = "final";
const FINAL_LEN: usize = FINAL.len();
const CHILD_OFFSET: u16 = 40;

impl TruncateWithEllipsis for String {
    fn ellipsis(&self, start_pos: u16, max_pos: u16) -> String {
        self.as_str().ellipsis(start_pos, max_pos)
    }
}
impl TruncateWithEllipsis for &str {
    /// Truncate the string, adding an ellipsis at the end to make the truncation clear.
    /// This is designed to be used with the TUI, hence the strange arguments.
    ///
    /// # Arguments
    ///
    /// * `start_pos` - The position in the TUI at which the string starts.
    /// * `max` - The maximum possible position in the TUI for the string to reach.
    ///
    fn ellipsis(&self, start_pos: u16, max_pos: u16) -> String {
        let max_len: usize = max_pos.saturating_sub(start_pos) as usize;
        if max_len <= ELLIPSIS_LEN {
            return ELLIPSIS.into();
        }

        let len = self.chars().count();
        if len <= max_len {
            return self.to_string();
        }
        let split_at_index = max_len - ELLIPSIS_LEN;

        // Most terminal commands are ASCII, so keep this simple and fast in those cases.
        //
        // NOTE: In the case of non-ASCII strings, 'split_at_checked' will return without an error
        // for some 'split_at_index' values, resulting in the truncated string being shorter than expected.
        // This is currently intentional as a minor sacrifice in the edge case.
        let start = match self.split_at_checked(split_at_index) {
            Some((s, _)) => s,
            None => {
                // In the rare case that a terminal command is not ASCII, we handle with the slower
                // implementation.
                let mut boundary = 0;

                for (i, c) in self.chars().enumerate() {
                    boundary += c.len_utf8();

                    if i >= split_at_index {
                        break;
                    }
                }

                let (start, _) = self.split_at(boundary);
                start
            }
        };

        format!("{start}{ELLIPSIS}")
    }
}

#[test]
fn test_truncate_utf8() {
    let t = "𓀂𓀂𓅮𓀂𓅮𓀂𓅮𓀂𓀂𓅮𓀂𓅮𓀂𓅮𓀂𓅮𓅮𓅮𓀂𓅮𓀂𓅮𓀂𓅮";

    for i in 0..t.chars().count() + 4 {
        let e = t.ellipsis(0, i as u16);
        eprintln!("{e:?}\t{}", e.chars().count());
    }

    let t = "hello";
    assert_eq!("...", t.ellipsis(0, 0));
    assert_eq!("...", t.ellipsis(0, 1));
    assert_eq!("...", t.ellipsis(0, 2));
    assert_eq!("...", t.ellipsis(0, 3));
    assert_eq!("h...", t.ellipsis(0, 4));
    assert_eq!("hello", t.ellipsis(0, 5));

    assert_eq!("", "".ellipsis(0, 7));
    assert_eq!("t", "t".ellipsis(0, 7));
    assert_eq!("tr", "tr".ellipsis(0, 7));
    assert_eq!("tro", "tro".ellipsis(0, 7));
    assert_eq!("troy", "troy".ellipsis(0, 7));
}

trait KeyBuffer {
    /// Remove all invalid sequences.
    fn strip_invalid(&mut self);

    /// Display in a format useful for the TUI.
    fn display(&self) -> String;

    /// Total length (in chars) of the sequences when displayed.
    fn total_visual_length(&self) -> usize;
}
impl KeyBuffer for Vec<CharWithModifiersAndValidity> {
    fn strip_invalid(&mut self) {
        *self = self.iter().filter(|d| d.valid).copied().collect();
    }

    fn display(&self) -> String {
        self.iter().map(|c| c.char.str_short()).join("")
    }

    fn total_visual_length(&self) -> usize {
        self.iter().map(|c| c.char.visual_length()).sum()
    }
}

pub struct Tui<'a> {
    key_buffer: Vec<CharWithModifiersAndValidity>,
    term: termion::screen::AlternateScreen<termion::raw::RawTerminal<File>>,
    tmpfile: String,
    node: &'a Node,
    parents: Vec<&'a Node>,
    /// Number of invalid keys.
    invalid_count: usize,
    /// Whether an invalid sequence is currently entered.
    invalid: bool,
    abbr: bool,
}

struct NodeMetadata<'a> {
    node: &'a Node,
    pos: (u16, u16),
    length: u16,
}

impl<'a> Tui<'a> {
    pub fn new(
        term: termion::screen::AlternateScreen<termion::raw::RawTerminal<File>>,
        tmpfile: String,
        node: &'a Node,
        abbr: bool,
    ) -> Self {
        Tui {
            key_buffer: vec![],
            term,
            tmpfile,
            node,
            parents: vec![],
            invalid_count: 0,
            invalid: false,
            abbr,
        }
    }
    pub fn run(mut self, stdin: std::io::Stdin) {
        self.write();

        let mut cancelled = false;
        let mut command = None;

        for c in stdin.keys() {
            let mut key = None;

            match c.as_ref().unwrap() {
                Key::Char('\n') | Key::Char(' ') | Key::Ctrl('M') => {
                    if let Some(ref cmd) = self.node.command {
                        command = Some(cmd);
                        break;
                    }
                }
                Key::Ctrl('c') | Key::Esc => {
                    cancelled = true;
                    break;
                }
                Key::Backspace => {
                    if self.key_buffer.is_empty() {
                        continue;
                    }
                    let last = self.key_buffer.pop();

                    if self.invalid_count == 0 {
                        self.node = self.parents.pop().unwrap();
                    }
                    if let Some(l) = last {
                        self.invalid_count =
                            self.invalid_count.saturating_sub(l.char.visual_length());
                    }
                    self.write();
                }
                Key::Ctrl(any) => {
                    key = modifier_or_fallback(CharWithModifiers::Ctrl(*any), &self.node.children);
                }
                Key::Alt(any) => {
                    key = modifier_or_fallback(CharWithModifiers::Alt(*any), &self.node.children);
                }
                Key::Char(any) => {
                    key = Some((*any).into());
                }
                _ => continue,
            }

            if let Some(key) = key {
                let valid = if let Some(node) = self.node.children.get(&key) {
                    if self.invalid {
                        self.invalid_count = 0;
                        self.invalid = false;
                        self.key_buffer.strip_invalid();
                    }
                    self.parents.push(self.node);
                    self.node = node;

                    if self.node.children.is_empty() && self.node.command.is_some() {
                        command = self.node.command.as_ref();
                        break;
                    }
                    true
                } else {
                    self.invalid_count += key.visual_length();
                    self.invalid = true;
                    false
                };

                self.key_buffer
                    .push(CharWithModifiersAndValidity { char: key, valid });
                self.write();
            }
        }

        if let Some(cmd) = command
            && !cancelled
        {
            let mut file = File::create(self.tmpfile).unwrap();

            file.write_all(cmd.as_bytes())
                .expect("Failed to write all output to tmpfile");

            file.flush().expect("Failed to flush to tmpfile");
        }

        write!(self.term, "{}", termion::cursor::Show).unwrap();
        write!(self.term, "{}", termion::clear::All).unwrap();
        self.term.flush().unwrap();
        self.term.suspend_raw_mode().unwrap();

        drop(self.term);
        std::process::exit(0);
    }

    pub fn write(&mut self) {
        let (term_width, term_height) = termion::terminal_size().unwrap();
        let compact_mode = term_width < 80;

        let width_first: u16 = 30;
        let mut height = 0;

        let space_char: CharWithModifiers = ' '.into();

        let current: String = self.key_buffer.display();

        let (normal_text, red_text) =
            current.split_at(self.key_buffer.total_visual_length() - self.invalid_count);

        // Draw the prompt with the current text.
        write!(
            self.term,
            "{}{}{}Type here: {}{}",
            termion::cursor::Goto(1, width_first),
            termion::clear::BeforeCursor,
            termion::cursor::Goto(1, 1),
            if !self.key_buffer.is_empty() {
                normal_text
            } else {
                "<waiting>"
            },
            red_text.red(),
        )
        .unwrap();

        let mut keys: Vec<&CharWithModifiers> = self.node.children.keys().collect();

        keys.sort_by(|a, b| {
            let alpha_cmp = a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase());

            if alpha_cmp == std::cmp::Ordering::Equal {
                a.is_uppercase().cmp(&b.is_uppercase())
            } else {
                alpha_cmp
            }
        });
        if self.node.command.is_some() {
            keys.insert(0, &space_char);
        }

        let mut values = self.node.children.clone();
        values.insert(' '.into(), self.node.clone());

        let mut nodes_drawn = vec![];

        // Draw the commands available.
        for (index, key) in keys.iter().enumerate() {
            let index = index as u16;
            let value = values.get(key).unwrap();
            height += 1;

            let num_subcommands = value.children.len();

            let command = value
                .command
                .as_ref()
                .unwrap_or(&format!(
                    "{} subcommand{}",
                    num_subcommands,
                    if num_subcommands == 1 { ' ' } else { 's' }
                ))
                .ellipsis(width_first, width_first + CHILD_OFFSET - 10);

            let only_subcommands = CharWithModifiers::Unmodified(' ') == **key;

            if !only_subcommands {
                nodes_drawn.push(NodeMetadata {
                    pos: (width_first, index + 1),
                    node: value,
                    length: command.len() as u16 + 6,
                });
            }

            let final_no_children =
                (value.children.is_empty() && value.command.is_some()) || only_subcommands;

            let spacing = if !final_no_children {
                " ".repeat(FINAL_LEN)
            } else {
                "".into()
            };

            write!(
                self.term,
                "{}{}{}{} | {} {}",
                termion::cursor::Goto(width_first, 1 + index),
                termion::clear::UntilNewline,
                spacing,
                match final_no_children {
                    true => FINAL.on_bright_blue().white().to_string(),
                    false => "".into(),
                },
                match only_subcommands {
                    true => "<space>".to_string().green(),
                    _ => key.str_short().green(),
                },
                command,
            )
            .unwrap();
        }

        if !compact_mode {
            let mut last_child_offset = 0u16;
            let mut pathfind_index = 1u16;

            // Clear any straggling trees of child nodes.
            write!(
                self.term,
                "{}{}",
                termion::cursor::Goto(width_first, 1 + keys.len() as u16),
                termion::clear::AfterCursor
            )
            .unwrap();

            let mut nodes_with_flat_children: Vec<(&NodeMetadata, Vec<&Node>)> = nodes_drawn
                .iter()
                .map(|it| {
                    let base = &it.node.command;

                    let mut children: Vec<&Node> = it
                        .node
                        .children
                        .values()
                        .flat_map(|it| {
                            if it.children.is_empty() {
                                vec![it]
                            } else {
                                let mut values: Vec<&Node> = it.children.values().collect();

                                values = values
                                    .iter()
                                    .flat_map(|c| {
                                        if c.children.is_empty() {
                                            vec![*c]
                                        } else {
                                            let mut m: Vec<&Node> = c.children.values().collect();
                                            m.insert(0, c);
                                            m
                                        }
                                    })
                                    .collect();

                                values
                            }
                        })
                        .filter(|it| it.command.is_some())
                        .collect();

                    children.sort_by(|a, b| {
                        if a.command.is_none() {
                            return Ordering::Greater;
                        } else if b.command.is_none() {
                            return Ordering::Less;
                        }
                        a.command.cmp(&b.command)
                    });

                    if it.node.children.iter().any(|v| {
                        v.1.command
                            .as_ref()
                            .and_then(|v| base.as_ref().map(|b| !v.contains(b)))
                            .unwrap_or(false)
                    }) {
                        children.insert(0, &it.node);
                    }

                    (it, children)
                })
                .collect();

            let mut num_lines = (0, 0);
            for (_, c) in nodes_with_flat_children.iter() {
                num_lines.0 += c.len();
                num_lines.1 += 1;
            }

            let reduction_needed = if num_lines.0 + num_lines.1 > term_height as usize {
                // Exceeded vectical space.
                Some(num_lines.1 + num_lines.0 - term_height as usize)
            } else {
                None
            };

            let placeholder = Node {
                command: Some(ELLIPSIS.to_string()),
                ..Default::default()
            };

            let mut reductions = 0;

            // Remove excess child nodes.
            if let Some(reduction_count) = reduction_needed {
                let mut index = 0;

                while reductions < reduction_count {
                    for (_, children) in nodes_with_flat_children.iter_mut() {
                        if index > 3 || children.len() > 1 {
                            let fudge_factor = if children.len() > 1 {
                                children.len() as f32 / 2f32
                            } else {
                                0f32
                            }
                            .ceil() as usize;

                            for _ in 0..fudge_factor {
                                if children.is_empty() {
                                    break;
                                }
                                children.pop();
                                reductions += 1;
                            }
                            children.push(&placeholder);
                        }
                    }

                    index += 1;
                }
            }

            // Draw the tree of child nodes.
            for (node_drawn, children) in nodes_with_flat_children.iter_mut() {
                let last_one = children.len().saturating_sub(1);

                if children.is_empty() {
                    last_child_offset = last_child_offset.saturating_sub(1);
                    continue;
                }

                // Draw each child in turn.
                for (index, cn) in children.iter().enumerate() {
                    let mut box_char = match index {
                        0 => box_drawing::light::DOWN_HORIZONTAL,
                        _ => box_drawing::light::VERTICAL,
                    };
                    if index == last_one {
                        if index > 0 {
                            box_char = box_drawing::light::UP_RIGHT;
                        } else {
                            box_char = box_drawing::light::VERTICAL_LEFT;
                        }
                    }

                    let child_pos = (
                        node_drawn.pos.0 + CHILD_OFFSET,
                        node_drawn.pos.1 + last_child_offset,
                    );
                    // Write the child command.
                    write!(
                        self.term,
                        "{} {} {}",
                        termion::cursor::Goto(child_pos.0, child_pos.1),
                        box_char.color(Color::BrightBlack),
                        cn.command
                            .as_ref()
                            .unwrap_or(&format!(
                                "{} subcommand(s): {:?}",
                                cn.children.len(),
                                cn.children
                            ))
                            .ellipsis(child_pos.0 + 3, term_width)
                            .color(Color::BrightBlack)
                    )
                    .unwrap();

                    last_child_offset += 1;

                    // Draw the line from the parent to the children.
                    if index == 0 {
                        let start = termion::cursor::Goto(
                            node_drawn.pos.0 + node_drawn.length + 5,
                            node_drawn.pos.1,
                        );
                        let turn_point =
                            termion::cursor::Goto(child_pos.0 - 1 - pathfind_index, child_pos.1);

                        let turn_point2 =
                            termion::cursor::Goto(child_pos.0 - 1 - pathfind_index, child_pos.1);

                        let is_inline = node_drawn.pos.1 == child_pos.1;

                        write!(
                            self.term,
                            "{}{}{}",
                            start,
                            box_drawing::light::HORIZONTAL
                                .repeat((turn_point.0.saturating_sub(start.0)) as usize)
                                .color(Color::BrightBlack),
                            if is_inline {
                                box_drawing::light::HORIZONTAL
                            } else {
                                box_drawing::light::DOWN_LEFT
                            }
                            .color(Color::BrightBlack),
                        )
                        .unwrap();

                        for row in start.1 + 1..child_pos.1 {
                            write!(
                                self.term,
                                "{}{}",
                                termion::cursor::Goto(turn_point.0, row),
                                box_drawing::light::VERTICAL.color(Color::BrightBlack),
                            )
                            .unwrap();
                        }
                        if !is_inline {
                            write!(
                                self.term,
                                "{}{}",
                                termion::cursor::Goto(turn_point.0, turn_point2.1),
                                box_drawing::light::UP_RIGHT.color(Color::BrightBlack),
                            )
                            .unwrap();
                        }

                        write!(
                            self.term,
                            "{}{}",
                            termion::cursor::Goto(turn_point2.0 + 1, turn_point2.1),
                            box_drawing::light::HORIZONTAL
                                .repeat((child_pos.0 - turn_point2.0) as usize)
                                .color(Color::BrightBlack),
                        )
                        .unwrap();

                        pathfind_index += 1;
                    }
                }
            }

            let (msg, color) = match self.abbr {
                true => (
                    "command will be expanded so you can add arguments",
                    "abbr".on_color(Color::Cyan),
                ),
                false => (
                    "command will be executed immediately",
                    "exec".on_color(Color::BrightRed),
                ),
            };

            // Draw the status line which shows which mode is active.
            write!(
                self.term,
                "{}{} mode: {}",
                termion::cursor::Goto(1, height + 10),
                color,
                msg,
            )
            .unwrap();

            // Draw the help text.
            write!(
                self.term,
                "{}{}",
                termion::cursor::Goto(1, height + 12),
                "Help: Press a key marked in green to select".yellow()
            )
            .unwrap();
            write!(
                self.term,
                "{}{}",
                termion::cursor::Goto(1, height + 13),
                "Help: Press Esc/Ctrl-c to cancel and exit".yellow(),
            )
            .unwrap();
            write!(self.term, "{}", termion::cursor::Hide,).unwrap();
        }

        self.term.flush().unwrap();
    }
}

/// Returns the modifier key if the modifier is present as a valid option, but if not, and if the unmodified key is
/// valid, returns the unmodified key.
fn modifier_or_fallback(
    key: CharWithModifiers,
    children: &std::collections::HashMap<
        CharWithModifiers,
        Node,
        std::hash::BuildHasherDefault<fnv::FnvHasher>,
    >,
) -> Option<CharWithModifiers> {
    if !children.contains_key(&key) {
        let unmodified = (*key).into();

        if children.contains_key(&unmodified) {
            return Some(unmodified);
        }
    }

    Some(key)
}
