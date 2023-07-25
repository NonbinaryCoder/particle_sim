use std::{borrow::Cow, fmt::Write, mem};

use unicode_width::UnicodeWidthStr;

use super::parsing::Position;

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
}

impl Diagnostics {
    pub fn init() -> Self {
        Self {
            diagnostics: Vec::new(),
            errored: false,
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.errored |= diagnostic.level == Level::Warn;
        self.diagnostics.push(diagnostic);
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

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn print_to_console(&self, files: &[(String, Vec<u8>)]) {
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
                let (name, file) = &files[position.file as usize];
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
        // Writing to a string never fails.
        let _ = write!(self.diagnostic.text.to_mut(), ": {}", context);
        self
    }

    pub fn quoted_context(mut self, context: impl std::fmt::Display) -> Self {
        // Writing to a string never fails.
        let _ = write!(self.diagnostic.text.to_mut(), ": \"{}\"", context);
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
