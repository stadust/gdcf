use crate::core::{
    backend::{Database, Error},
    FromSql,
};
use gdcf_model::{
    level::{Featured, LevelLength, LevelRating, Password},
    song::MainSong,
    GameVersion,
};

impl<DB: Database> FromSql<DB> for LevelRating
where
    String: FromSql<DB>,
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
    where
        Self: Sized,
    {
        String::from_sql(sql).map(LevelRating::from)
    }
}

impl<DB: Database> FromSql<DB> for &'static MainSong
where
    u8: FromSql<DB>,
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
    where
        Self: Sized,
    {
        u8::from_sql(sql).map(Self::from)
    }
}

impl<DB: Database> FromSql<DB> for GameVersion
where
    u8: FromSql<DB>,
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
    where
        Self: Sized,
    {
        u8::from_sql(sql).map(GameVersion::from)
    }
}

impl<DB: Database> FromSql<DB> for LevelLength
where
    String: FromSql<DB>,
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
    where
        Self: Sized,
    {
        String::from_sql(sql).map(LevelLength::from)
    }
}

impl<DB: Database> FromSql<DB> for Featured
where
    i32: FromSql<DB>,
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
    where
        Self: Sized,
    {
        i32::from_sql(sql).map(Featured::from)
    }
}

impl<DB: Database> FromSql<DB> for Password
where
    Option<String>: FromSql<DB>,
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
    where
        Self: Sized,
    {
        let pass = Option::<String>::from_sql(sql)?;

        Ok(match pass {
            None => Password::NoCopy,
            Some(pass) =>
                if pass == "1" {
                    Password::FreeCopy
                } else {
                    Password::PasswordCopy(pass.to_string())
                },
        })
    }
}
