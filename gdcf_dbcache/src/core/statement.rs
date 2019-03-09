use crate::core::{backend::Database, AsSql};
use joinery::Joinable;

#[derive(Debug)]
pub enum StatementPart {
    Static(String),
    Placeholder,
}

#[derive(Debug, Default)]
pub struct PreparedStatement {
    parts: Vec<StatementPart>,
}

impl PreparedStatement {
    pub fn placeholder() -> PreparedStatement {
        PreparedStatement {
            parts: vec![StatementPart::Placeholder],
        }
    }

    pub fn concat(&mut self, mut other: PreparedStatement) {
        self.parts.append(&mut other.parts)
    }

    pub fn to_statement(&self, placeholder_fmt: fn(usize) -> String) -> String {
        let mut idx = 0;

        self.parts
            .iter()
            .map(move |part| {
                match part {
                    StatementPart::Static(string) => string.to_string(),
                    StatementPart::Placeholder => {
                        idx += 1;
                        placeholder_fmt(idx)
                    },
                }
            })
            .join_with(" ")
            .to_string()
    }

    pub fn pop(&mut self) -> Option<StatementPart> {
        self.parts.pop()
    }
}

pub type Preparation<'a, DB> = (PreparedStatement, Vec<&'a dyn AsSql<DB>>);

pub trait Prepare<DB: Database>: Default {
    fn with_static<S: Into<String>>(self, s: S) -> Self;
    fn with(self, other: Self) -> Self;

    fn unprepared(&self) -> String;
}

impl<'a, DB: Database> Prepare<DB> for (PreparedStatement, Vec<&'a dyn AsSql<DB>>) {
    fn with_static<S: Into<String>>(mut self, s: S) -> Self {
        self.0.parts.push(StatementPart::Static(s.into()));
        self
    }

    fn with(mut self, mut other: Self) -> Self {
        self.0.concat(other.0);
        self.1.append(&mut other.1);

        self
    }

    fn unprepared(&self) -> String {
        let mut idx = 0;

        self.0
            .parts
            .iter()
            .map(move |part| {
                match part {
                    StatementPart::Static(string) => string.clone(),
                    StatementPart::Placeholder => {
                        let raw = self.1[idx].as_sql().to_string();
                        idx += 1;
                        raw
                    },
                }
            })
            .join_with(" ")
            .to_string()
    }
}

impl<T> From<T> for StatementPart
where
    T: Into<String>,
{
    fn from(t: T) -> Self {
        StatementPart::Static(t.into())
    }
}

impl<T> From<T> for PreparedStatement
where
    T: Into<String>,
{
    fn from(t: T) -> Self {
        let mut stmt = PreparedStatement::default();
        stmt.parts.push(t.into().into());
        stmt
    }
}
