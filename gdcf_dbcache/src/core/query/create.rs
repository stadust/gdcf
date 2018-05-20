use core::AsSql;
use core::backend::Database;
use core::query::Query;
use core::query::QueryPart;
use core::table::Field;
use core::types::Type;

#[derive(Debug)]
pub(crate) struct Create<'a, DB: Database + 'a> {
    pub(crate) name: &'a str,
    pub(crate) ignore_if_exists: bool,
    pub(crate) columns: Vec<Column<'a, DB>>,
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
    pub(crate) name: &'a str,
    pub(crate) sql_type: Box<Type<'a, DB>>,
    pub(crate) constraints: Vec<Box<Constraint<'a, DB>>>,
}

impl<'a, DB: Database + 'a> Column<'a, DB> {
    pub(crate) fn new<T: Type<'a, DB> + 'static>(name: &'a str, sql_type: T) -> Column<'a, DB> {
        Column {
            name,
            sql_type: Box::new(sql_type),
            constraints: Vec::new(),
        }
    }

    pub(crate) fn primary(self) -> Self
        where
            PrimaryKeyConstraint<'a>: Constraint<'a, DB> + 'static
    {
        self.constraint(PrimaryKeyConstraint::default())
    }

    pub(crate) fn unique(self) -> Self
        where
            UniqueConstraint<'a>: Constraint<'a, DB> + 'static
    {
        self.constraint(UniqueConstraint::default())
    }

    pub(crate) fn not_null(self) -> Self
        where
            NotNullConstraint<'a>: Constraint<'a, DB> + 'static
    {
        self.constraint(NotNullConstraint::default())
    }

    pub(crate) fn default(self, default: &'a AsSql<DB>) -> Self
        where
            DefaultConstraint<'a, DB>: Constraint<'a, DB> + 'static
    {
        self.constraint(DefaultConstraint::new(None, default))
    }

    pub(crate) fn foreign_key(self, references: &'a Field) -> Self
        where
            ForeignKeyConstraint<'a>: Constraint<'a, DB> + 'static
    {
        self.constraint(ForeignKeyConstraint::new(None, references))
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
pub(crate) struct NotNullConstraint<'a>(pub(crate) Option<&'a str>);

#[derive(Debug, Default)]
pub(crate) struct UniqueConstraint<'a>(pub(crate) Option<&'a str>);

#[derive(Debug, Default)]
pub(crate) struct PrimaryKeyConstraint<'a>(pub(crate) Option<&'a str>);

#[derive(Debug)]
pub(crate) struct ForeignKeyConstraint<'a> {
    name: Option<&'a str>,
    references: &'a Field,
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

impl<'a> ForeignKeyConstraint<'a> {
    pub(crate) fn new(name: Option<&'a str>, references: &'a Field) -> ForeignKeyConstraint<'a> {
        ForeignKeyConstraint {
            name,
            references,
        }
    }
}

if_query_part!(NotNullConstraint<'a>, Constraint<'a, DB>);
if_query_part!(UniqueConstraint<'a>, Constraint<'a, DB>);
if_query_part!(PrimaryKeyConstraint<'a>, Constraint<'a, DB>);
if_query_part!(ForeignKeyConstraint<'a>, Constraint<'a, DB>);
if_query_part!(DefaultConstraint<'a, DB>, Constraint<'a, DB>);

impl<'a, DB: Database + 'a> Query<'a, DB> for Create<'a, DB>
    where
        Create<'a, DB>: QueryPart<'a, DB> {}