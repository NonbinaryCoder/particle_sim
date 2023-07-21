use std::{borrow::Cow, mem};

use bevy::prelude::{error, warn};

use super::parsing::Position;

#[derive(Debug)]
pub struct Diagnostics(Vec<Diagnostic>);

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

impl Diagnostics {
    pub fn init() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        match diagnostic.level {
            Level::Warn => warn!("{}", diagnostic.text),
            Level::Error => error!("{}", diagnostic.text),
        }
        self.0.push(diagnostic);
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
