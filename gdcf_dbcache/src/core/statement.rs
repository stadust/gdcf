use core::AsSql;
use core::backend::Database;
use core::query::QueryPart;
use gdcf::ext::Join;

#[derive(Debug)]
pub enum StatementPart {
    Static(String),
    Placeholder,
}

#[derive(Debug)]
pub struct PreparedStatement {
    parts: Vec<StatementPart>
}

impl PreparedStatement {
    pub fn new(parts: Vec<StatementPart>) -> PreparedStatement {
        PreparedStatement { parts }
    }

    pub fn concat(&mut self, mut other: PreparedStatement) {
        self.parts.append(&mut other.parts)
    }

    pub fn concat_on<T: Into<StatementPart>>(&mut self, other: PreparedStatement, on: T) {
        self.append(on);
        self.concat(other)
    }

    pub fn prepend<T: Into<StatementPart>>(&mut self, part: T) {
        self.parts.insert(0, part.into())
    }

    pub fn append<T: Into<StatementPart>>(&mut self, part: T) {
        self.parts.push(part.into())
    }

    pub fn pop(&mut self) -> Option<StatementPart> {
        self.parts.pop()
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
}

impl<T> From<T> for StatementPart
    where
        T: ToString
{
    fn from(t: T) -> Self {
        StatementPart::Static(t.to_string())
    }
}

impl<T> From<T> for PreparedStatement
    where
        T: ToString
{
    fn from(t: T) -> Self {
        PreparedStatement::new(vec![t.into()])
    }
}