pub(crate) enum StatementPart {
    Static(String),
    Placeholder,
}

pub(crate) struct PreparedStatement {
    parts: Vec<StatementPart>
}

impl PreparedStatement {
    pub(crate) fn new(parts: Vec<StatementPart>) -> PreparedStatement {
        PreparedStatement { parts }
    }

    pub(crate) fn concat(&mut self, mut other: PreparedStatement) {
        self.parts.append(&mut other.parts)
    }

    pub(crate) fn concat_on<T: Into<StatementPart>>(&mut self, other: PreparedStatement, on: T) {
        self.append(on);
        self.concat(other)
    }

    pub(crate) fn prepend<T: Into<StatementPart>>(&mut self, part: T) {
        self.parts.insert(0, part.into())
    }

    pub(crate) fn append<T: Into<StatementPart>>(&mut self, part: T) {
        self.parts.push(part.into())
    }

    pub(crate) fn to_statement(&self, placeholder_fmt: fn(usize) -> String) -> String {
        let mut idx = 0;
        let mut stmt = String::new();

        for part in &self.parts {
            idx += 1;

            match part {
                StatementPart::Static(string) => stmt = format!("{} {}", stmt, string),
                StatementPart::Placeholder => stmt = format!("{} {}", stmt, placeholder_fmt(idx))
            }
        }

        stmt
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