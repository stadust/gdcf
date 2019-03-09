use itertools::Itertools;
use proc_macro::{Delimiter, Group, Ident, Spacing, TokenStream, TokenTree};
use std::iter::FromIterator;

#[derive(Debug)]
pub(super) struct Create {
    table: Ident,
    columns: Vec<Column>,
}

#[derive(Debug)]
pub(super) struct Column {
    name: Ident,
    col_type: TokenStream,
    constraints: Vec<Constraint>,
}

#[derive(Debug)]
pub(super) enum Constraint {
    NotNull,
    Unique,
    Primary,
    Default { ty: TokenStream, val: TokenStream },
}

impl PartialEq for Constraint {
    fn eq(&self, other: &Constraint) -> bool {
        match self {
            Constraint::NotNull =>
                match other {
                    Constraint::NotNull => true,
                    _ => false,
                },
            Constraint::Unique =>
                match other {
                    Constraint::Unique => true,
                    _ => false,
                },
            Constraint::Primary =>
                match other {
                    Constraint::Primary => true,
                    _ => false,
                },
            Constraint::Default { ty, .. } =>
                match other {
                    Constraint::Default { ty: ty2, .. } => ty2.to_string() == ty.to_string(),
                    _ => false,
                },
        }
    }
}

impl Create {
    pub(super) fn parse(ts: TokenStream) -> Create {
        let mut iter = ts.into_iter();

        let table_name = ident!(iter);
        joint_punct!(iter, '=');
        alone_punct!(iter, '>');

        let body = any_group!(iter).stream();

        iter = body.into_iter();

        let mut columns = Vec::new();

        'outer: loop {
            let field = ident!(iter);

            alone_punct!(iter, ':');

            let (ty, mut next) = parse_ty(&mut iter);
            let mut column = Column {
                name: field,
                col_type: ty,
                constraints: Vec::new(),
            };

            loop {
                match next {
                    Some(TokenTree::Punct(ref punct)) if punct.as_char() == ',' => break,
                    Some(TokenTree::Ident(ref ident)) => {
                        let cons = ident.to_string();
                        let constraint = match cons.as_ref() {
                            "NotNull" => Constraint::NotNull,
                            "Primary" => Constraint::Primary,
                            "Unique" => Constraint::Unique,
                            "Default" => parse_default(&mut iter),
                            crap => panic!("Unexpected constraint: {}", crap),
                        };

                        column.constraints.push(constraint);
                    },
                    Some(crap) => panic!("Expected ident or ',', got {:?}", crap),
                    None => break 'outer columns.push(column),
                }

                next = iter.next();
            }

            columns.push(column);
        }

        Create {
            table: table_name,
            columns,
        }
    }

    pub(super) fn generate(&self) -> TokenStream {
        let mut streams = Vec::<TokenStream>::new();

        streams.push(stream! {
            "use crate::core::types::*;".parse().unwrap(),
            "use crate::core::query::create::*;".parse().unwrap(),
            "use crate::core::SqlExpr;".parse().unwrap(),
            "use crate::core::backend::Database;".parse().unwrap()
        });

        streams.push(
            "pub fn create<'a, DB: Database + 'a>() -> Create<'a, DB> where"
                .to_string()
                .parse()
                .unwrap(),
        );

        // Rust is awesome and we can actually keep a Vector of
        // references and it wont compare pointers but use the pointed-at
        // object's PartialEq impl. Nice!
        let mut constraints = Vec::new();
        let mut types = Vec::new();

        streams.push(
            self.columns
                .iter()
                .flat_map(|c| c.constraints.iter())
                .filter(|c| {
                    if constraints.contains(c) {
                        false
                    } else {
                        constraints.push(c);
                        true
                    }
                })
                .map(Constraint::where_constraint)
                .chain(
                    self.columns
                        .iter()
                        .map(|c| c.col_type.clone())
                        .filter(|c| {
                            if types.contains(&c.to_string()) {
                                false
                            } else {
                                types.push(c.to_string());
                                true
                            }
                        })
                        .map(|c| {
                            stream! {
                                c,
                                ": Type<DB>".parse().unwrap()
                            }
                        }),
                )
                .intersperse(",".parse().unwrap())
                .collect(),
        );

        let body = self
            .columns
            .iter()
            .map(|col| {
                let constraints = col
                    .constraints
                    .iter()
                    .map(|cons| {
                        stream! {
                            ".constraint".parse().unwrap(),
                            TokenTree::Group(Group::new(Delimiter::Parenthesis, cons.generate())).into()
                        }
                    })
                    .collect();
                let inner = Group::new(
                    Delimiter::Brace,
                    stream! {
                        "let ty:".parse().unwrap(),
                        col.col_type.clone(),
                        " = Default::default(); ty".parse().unwrap()
                    },
                );
                let args = Group::new(
                    Delimiter::Parenthesis,
                    stream! {
                        format!("{}.name(),", col.name).parse().unwrap(),
                        TokenTree::Group(inner).into()
                    },
                );
                let args = Group::new(
                    Delimiter::Parenthesis,
                    stream! {
                        "Column::new".parse().unwrap(),
                        TokenTree::Group(args).into(),
                        constraints
                    },
                );

                stream! {
                    ".with_column".parse().unwrap(),
                    TokenTree::Group(args).into()
                }
            })
            .collect();

        streams.push(
            TokenTree::Group(Group::new(
                Delimiter::Brace,
                stream! {
                    "table.create()".to_string().parse().unwrap(),
                    body
                },
            ))
            .into(),
        );

        TokenStream::from_iter(streams)
    }
}

impl Constraint {
    fn where_constraint(&self) -> TokenStream {
        match self {
            Constraint::Unique => "UniqueConstraint<'a>: Constraint<DB> + 'static".parse().unwrap(),
            Constraint::NotNull => "NotNullConstraint<'a>: Constraint<DB> + 'static".parse().unwrap(),
            Constraint::Primary => "PrimaryKeyConstraint<'a>: Constraint<DB> + 'static".parse().unwrap(),
            Constraint::Default { ty, .. } =>
                stream! {
                    "DefaultConstraint<'a, DB>: Constraint<DB> + 'static,".parse().unwrap(),
                    ty.clone(),
                    ": SqlExpr<DB>".parse().unwrap()
                },
        }
    }

    fn generate(&self) -> TokenStream {
        match self {
            Constraint::Unique => "UniqueConstraint::default()".parse().unwrap(),
            Constraint::NotNull => "NotNullConstraint::default()".parse().unwrap(),
            Constraint::Primary => "PrimaryKeyConstraint::default()".parse().unwrap(),
            Constraint::Default { val, .. } => {
                stream! {
                    "DefaultConstraint::new".parse().unwrap(),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, stream! {
                        "None,".parse().unwrap(),
                        val.clone()
                    })).into()
                }
            },
        }
    }
}

pub(crate) fn parse_ty(iter: &mut impl Iterator<Item = TokenTree>) -> (TokenStream, Option<TokenTree>) {
    let mut tts = vec![TokenTree::Ident(ident!(iter))];
    let next = iter.next();

    match next {
        Some(TokenTree::Punct(p)) =>
            if p.as_char() == '<' {
                tts.push(TokenTree::Punct(p));

                loop {
                    let (inner, end) = parse_ty(iter);

                    tts.extend(inner);

                    match end {
                        Some(TokenTree::Punct(p)) => {
                            let c = p.as_char();
                            tts.push(TokenTree::Punct(p));
                            match c {
                                '>' => break (TokenStream::from_iter(tts), iter.next()),
                                ',' => continue,
                                _ => panic!("Expected '>' or ',', got {:?}", c),
                            }
                        },
                        Some(end) => panic!("Expected '>' or ',', got {:?}", end),
                        None => panic!("Expected '>' or ',', got end-of-stream"),
                    }
                }
            } else {
                (TokenStream::from_iter(tts), Some(TokenTree::Punct(p)))
            },
        next => (TokenStream::from_iter(tts), next),
    }
}

fn parse_default(iter: &mut impl Iterator<Item = TokenTree>) -> Constraint {
    alone_punct!(iter, '<');
    let (ty, next) = parse_ty(iter);

    if let Some(TokenTree::Punct(ref punct)) = next {
        if punct.as_char() == '>' {
            let grp = any_group!(iter);

            return Constraint::Default { ty, val: grp.stream() }
        }
    }

    panic!("Expected '>', got {:?}", next);
}
