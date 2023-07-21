use std::{borrow::Cow, mem};

use unicode_width::UnicodeWidthStr;

use super::parsing::{FileId, Position};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    level: Level,
    position: Option<Position>,
    text: Cow<'static, str>,
}

impl Diagnostic {
    const NULL: Self = Self {
        level: Level::Warn,
        position: None,
        text: Cow::Borrowed(""),
    };
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    #[default]
    Warn,
    Error,
}

#[derive(Debug)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
    errored: bool,
    files: Vec<(String, Vec<u8>)>,
}

impl Diagnostics {
    pub fn init() -> Self {
        Self {
            diagnostics: Vec::new(),
            errored: false,
            files: Vec::new(),
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.errored |= diagnostic.level == Level::Warn;
        self.diagnostics.push(diagnostic);
    }

    /// The id the next file added to this will have.
    pub fn next_id(&self) -> FileId {
        self.files.len() as FileId
    }

    /// Adds a file to be tracked by this.
    ///
    /// Tracked files are used to add context to errors.
    pub fn add_file(&mut self, name: String, file: Vec<u8>) {
        self.files.push((name, file));
    }

    pub fn warn(&mut self, text: impl Into<Cow<'static, str>>) -> DiagnosticBuilder {
        DiagnosticBuilder {
            collection: self,
            diagnostic: Diagnostic {
                level: Level::Warn,
                position: None,
                text: text.into(),
            },
        }
    }

    pub fn error(&mut self, text: impl Into<Cow<'static, str>>) -> DiagnosticBuilder {
        DiagnosticBuilder {
            collection: self,
            diagnostic: Diagnostic {
                level: Level::Error,
                position: None,
                text: text.into(),
            },
        }
    }

    pub fn has_errored(&self) -> bool {
        self.errored
    }

    pub fn print_to_console(&self) {
        for diagnostic in &self.diagnostics {
            let Diagnostic {
                level,
                position,
                text,
            } = diagnostic;
            match level {
                Level::Warn => print!("warning: "),
                Level::Error => print!("error: "),
            }
            println!("{text}");

            if let Some(position) = position {
                let (name, file) = &self.files[position.file as usize];
                if let Ok(file) = std::str::from_utf8(file) {
                    let (line, col, line_text) = line_col(file, position.index);
                    println!("  --> {name}:{line}:{col}");
                    println!("   |");
                    println!("{line:<3}| {line_text}");
                    print!("   | ");
                    for _ in 1..col {
                        print!(" ");
                    }
                    println!("^");
                    println!("   |");
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct DiagnosticBuilder<'a> {
    collection: &'a mut Diagnostics,
    diagnostic: Diagnostic,
}

impl<'a> DiagnosticBuilder<'a> {
    pub fn position(mut self, position: Position) -> Self {
        self.diagnostic.position = Some(position);
        self
    }

    pub fn context(mut self, context: impl std::fmt::Display) -> Self {
        self.diagnostic
            .text
            .to_mut()
            .push_str(&format!(": {}", context));
        self
    }

    pub fn w(self) {}
}

impl<'a> Drop for DiagnosticBuilder<'a> {
    fn drop(&mut self) {
        self.collection
            .add(mem::replace(&mut self.diagnostic, Diagnostic::NULL));
    }
}

fn line_col(text: &str, mut position: u32) -> (u32, u32, &str) {
    let mut index = 1;
    for line in text.lines() {
        let len = UnicodeWidthStr::width(line);
        if len as u32 >= position {
            return (index, position + 1, line);
        } else {
            position -= len as u32 + 1;
            index += 1;
        }
    }
    (index, position, "")
}
