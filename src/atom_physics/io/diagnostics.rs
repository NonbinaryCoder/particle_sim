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

    fn description(&self, _buf: &mut dyn std::io::Write) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Diagnostics {
    diagnostics: Vec<Box<dyn Diagnostic>>,
    errored: bool,
}

impl Diagnostics {
    pub fn init() -> Self {
        Self {
            diagnostics: Vec::new(),
            errored: false,
        }
    }

    pub fn add(&mut self, diagnostic: impl Diagnostic + 'static) {
        self.errored |= diagnostic.level() == Level::Warn;
        self.diagnostics.push(Box::new(diagnostic));
    }

    pub fn has_errored(&self) -> bool {
        self.errored
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub(super) fn print_to_console(&self, _files: &IdMap<FileContents>) {
        for diagnostic in &self.diagnostics {
            dbg!(diagnostic);
        }
    }
}
