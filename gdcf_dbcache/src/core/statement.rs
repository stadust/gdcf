use core::AsSql;
use core::backend::Database;
use core::query::QueryPart;
use gdcf::ext::Join;

#[derive(Debug)]
pub enum StatementPart {
    Static(String),
    Placeholder,
}

#[derive(Debug, Default)]
pub struct PreparedStatement {
    parts: Vec<StatementPart>
}

impl PreparedStatement {
    pub fn concat(&mut self, mut other: PreparedStatement) {
        self.parts.append(&mut other.parts)
    }

    pub fn to_statement(&self, placeholder_fmt: fn(usize) -> String) -> String {
        let mut idx = 0;

        self.parts.iter()
            .map(|part| match part {
                StatementPart::Static(string) => string.to_string(),
                StatementPart::Placeholder => {
                    idx += 1;
                    placeholder_fmt(idx)
                }
            })
            .join(" ")
    }

    pub fn pop(&mut self) -> Option<StatementPart> {
        self.parts.pop()
    }
}

pub type Preparation<'a, DB> = (PreparedStatement, Vec<&'a AsSql<DB>>);

pub trait Prepare<DB: Database>: Default {
    fn with_static<S: Into<String>>(mut self, s: S) -> Self;
    fn with(mut self, mut other: Self) -> Self;
}

impl<'a, DB: Database> Prepare<DB> for (PreparedStatement, Vec<&'a AsSql<DB>>) {
    fn with_static<S: Into<String>>(mut self, s: S) -> Self {
        self.0.parts.push(StatementPart::Static(s.into()));
        self
    }

    fn with(mut self, mut other: Self) -> Self {
        self.0.concat(other.0);
        self.1.append(&mut other.1);

        self
    }
}

impl<T> From<T> for StatementPart
    where
        T: Into<String>
{
    fn from(t: T) -> Self {
        StatementPart::Static(t.into())
    }
}

impl<T> From<T> for PreparedStatement
    where
        T: Into<String>
{
    fn from(t: T) -> Self {
        let mut stmt = PreparedStatement::default();
        stmt.parts.push(t.into().into());
        stmt
    }
}