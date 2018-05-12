use core::AsSql;
use core::backend::Database;
use core::query::QueryPart;

#[derive(Debug)]
pub(crate) struct Create<'a, DB: Database + 'a> {
    name: &'a str,
    ignore_if_exists: bool,
    columns: Vec<Column<'a, DB>>,
}

impl<'a, DB: Database + 'a> Create<'a, DB> {
    pub(crate) fn new(name: &'a str) -> Create<'a, DB> {
        Create {
            name,
            ignore_if_exists: false,
            columns: Vec::new(),
        }
    }

    pub(crate) fn ignore_if_exists(mut self) -> Self {
        self.ignore_if_exists = true;
        self
    }

    pub(crate) fn with_column(mut self, col: Column<'a, DB>) -> Self {
        self.columns.push(col);
        self
    }
}

#[derive(Debug)]
pub(crate) struct Column<'a, DB: Database + 'a> {
    name: &'a str,
    type_name: &'a str,
    constraints: Vec<Box<Constraint<'a, DB>>>,
}

impl<'a, DB: Database + 'a> Column<'a, DB> {
    pub(crate) fn new(name: &'a str, type_name: &'a str) -> Column<'a, DB> {
        Column {
            name,
            type_name,
            constraints: Vec::new(),
        }
    }

    pub(crate) fn primary(mut self) -> Self
        where
            PrimaryKeyConstraint<'a>: Constraint<'a, DB> + 'static
    {
        self.constraint(PrimaryKeyConstraint::default())
    }

    pub(crate) fn unique(mut self) -> Self
        where
            UniqueConstraint<'a>: Constraint<'a, DB> + 'static
    {
        self.constraint(UniqueConstraint::default())
    }

    pub(crate) fn not_null(mut self) -> Self
        where
            NotNullConstraint<'a>: Constraint<'a, DB> + 'static
    {
        self.constraint(NotNullConstraint::default())
    }

    pub(crate) fn default(mut self, default: &'a AsSql<DB>) -> Self
        where
            DefaultConstraint<'a, DB>: Constraint<'a, DB> + 'static
    {
        self.constraint(DefaultConstraint::new(None, default))
    }

    pub(crate) fn constraint<Con: 'static>(mut self, constraint: Con) -> Self
        where
            Con: Constraint<'a, DB>
    {
        self.constraints.push(Box::new(constraint));
        self
    }
}

pub(crate) trait Constraint<'a, DB: Database + 'a>: QueryPart<'a, DB> {
    fn name(&'a self) -> Option<&'a str> {
        None
    }
}

#[derive(Debug, Default)]
pub(crate) struct NotNullConstraint<'a>(Option<&'a str>);

#[derive(Debug, Default)]
pub(crate) struct UniqueConstraint<'a>(Option<&'a str>);

#[derive(Debug, Default)]
pub(crate) struct PrimaryKeyConstraint<'a>(Option<&'a str>);

#[derive(Debug)]
pub(crate) struct ForeignKeyConstraint<'a> {
    name: Option<&'a str>,
    // TODO: rest
}

#[derive(Debug)]
pub(crate) struct DefaultConstraint<'a, DB: Database + 'a> {
    name: Option<&'a str>,
    default: &'a AsSql<DB>,
}

impl<'a, DB: Database + 'a> DefaultConstraint<'a, DB> {
    pub(crate) fn new(name: Option<&'a str>, default: &'a AsSql<DB>) -> DefaultConstraint<'a, DB> {
        DefaultConstraint {
            name,
            default,
        }
    }
}