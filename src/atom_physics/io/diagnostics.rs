use std::ops::{Deref, DerefMut};

use nom_locate::LocatedSpan;

use crate::atom_physics::id::IdMap;

use super::FileContents;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    #[default]
    Warn,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Warn => write!(f, "Warning"),
            Level::Error => write!(f, "Error"),
        }
    }
}

pub type Span<'a> = LocatedSpan<&'a str, FileId>;

pub type FileId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    file: FileId,
    offset: usize,
    line: u32,
    length: u16,
}

impl Position {
    /// `start..end`
    pub fn from_start_end(start: Span, end: Span) -> Position {
        debug_assert_eq!(
            start.extra, end.extra,
            "Nothing should cross file boundaries like this"
        );
        assert!(start.location_offset() <= end.location_offset());
        Self {
            file: start.extra,
            offset: start.location_offset(),
            line: start.location_line(),
            length: (end.location_offset() - start.location_offset()) as u16,
        }
    }

    /// Creates a new position containing `self` and `length` bytes before it.
    ///
    /// Will return incorrect result if the new start is no longer on the same
    /// line.
    pub fn extend_back_same_line(self, length: u16) -> Position {
        Position {
            offset: self.offset - length as usize,
            length: self.length + length,
            ..self
        }
    }

    /// Creates a new position containing `self` and `other`.
    ///
    /// May not work if `other` is before `self`.
    pub fn extend_to(self, other: Position) -> Position {
        debug_assert_eq!(
            self.file, other.file,
            "Nothing should cross file boundaries like this"
        );
        Position {
            file: self.file,
            offset: self.offset,
            line: self.line,
            length: (other.offset - self.offset) as u16 + other.length,
        }
    }

    pub fn position<T>(self, object: T) -> Positioned<T> {
        Positioned {
            object,
            position: self,
        }
    }

    pub fn char_inline(self, pos: usize) -> Position {
        debug_assert!(pos < self.length as usize);
        Position {
            offset: self.offset + pos,
            length: 1,
            ..self
        }
    }
}

impl<'a> From<Span<'a>> for Position {
    fn from(value: Span<'a>) -> Self {
        Self {
            file: value.extra,
            offset: value.location_offset(),
            line: value.location_line(),
            length: value.len().try_into().unwrap_or(u16::MAX),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Positioned<T> {
    pub object: T,
    pub position: Position,
}

impl<T> Deref for Positioned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<T> DerefMut for Positioned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}

impl<'a> From<Span<'a>> for Positioned<&'a str> {
    fn from(value: Span<'a>) -> Self {
        Self {
            object: *value,
            position: value.into(),
        }
    }
}

impl<T: PartialEq> PartialEq for Positioned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

impl<T: Eq> Eq for Positioned<T> {}

impl<T> Positioned<T> {
    #[cfg(test)]
    pub fn test_position(object: T) -> Positioned<T> {
        Positioned {
            object,
            position: Position {
                file: 0,
                offset: 0,
                line: 0,
                length: 1,
            },
        }
    }
}

pub trait Diagnostic: std::fmt::Debug {
    fn level(&self) -> Level;

    fn description(&self) -> String;
}

#[derive(Debug)]
pub struct Diagnostics {
    diagnostics: Vec<(Option<Position>, Box<dyn Diagnostic>)>,
    errored: bool,
}

impl Diagnostics {
    pub fn init() -> Self {
        Self {
            diagnostics: Vec::new(),
            errored: false,
        }
    }

    pub fn add(&mut self, position: Position, diagnostic: impl Diagnostic + 'static) {
        self.errored |= diagnostic.level() == Level::Error;
        self.diagnostics
            .push((Some(position), Box::new(diagnostic)));
    }

    pub fn add_positioned<T: Diagnostic + 'static>(&mut self, val: Positioned<T>) {
        self.add(val.position, val.object);
    }

    pub fn add_unpositioned(&mut self, diagnostic: impl Diagnostic + 'static) {
        self.errored |= diagnostic.level() == Level::Error;
        self.diagnostics.push((None, Box::new(diagnostic)));
    }

    pub fn has_errored(&self) -> bool {
        self.errored
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub(super) fn print_to_console(&self, files: &IdMap<FileContents>) {
        for (pos, diagnostic) in &self.diagnostics {
            print_diagnostic(files, *pos, &**diagnostic);
        }
    }
}

fn print_diagnostic(
    files: &IdMap<FileContents>,
    position: Option<Position>,
    diagnostic: &dyn Diagnostic,
) {
    println!("{}: {}", diagnostic.level(), diagnostic.description());
    if let Some(position) = position {
        let (file_name, FileContents(file_contents)) = &mut files.get_full(position.file).unwrap();
        let mut line_start_offset = 0;
        for line in file_contents.split_inclusive('\n') {
            let line_end_offset = line_start_offset + line.len();
            let line = line.trim();

            if position.offset < line_end_offset {
                let start_x = position.offset - line_start_offset;
                println!("  --> {file_name}:{}:{}", position.line, start_x);
                if (position.length as usize) <= line_end_offset - position.offset {
                    println!("{:<4}| {}", position.line, line);
                    print!("    | ");
                    for _ in 0..start_x {
                        print!(" ");
                    }
                    for _ in 0..position.length {
                        print!("^");
                    }
                    println!();
                }
                break;
            }

            line_start_offset = line_end_offset;
        }
    }
}
