use core::backend::Database;
use core::backend::Error;
use core::FromSql;
use gdcf::model::GameVersion;
use gdcf::model::level::Featured;
use gdcf::model::LevelLength;
use gdcf::model::LevelRating;
use gdcf::model::MainSong;
use std::str::FromStr;

impl<DB: Database> FromSql<DB> for LevelRating
    where
        String: FromSql<DB>
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
        where
            Self: Sized
    {
        String::from_sql(sql).map(LevelRating::from)
    }
}

impl<DB: Database> FromSql<DB> for &'static MainSong
    where
        u8: FromSql<DB>
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
        where
            Self: Sized
    {
        u8::from_sql(sql).map(Self::from)
    }
}

impl<DB: Database> FromSql<DB> for GameVersion
    where
        u8: FromSql<DB>
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
        where
            Self: Sized
    {
        u8::from_sql(sql).map(GameVersion::from)
    }
}

impl<DB: Database> FromSql<DB> for LevelLength
    where
        String: FromSql<DB>
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
        where
            Self: Sized
    {
        String::from_sql(sql).map(LevelLength::from)
    }
}

impl<DB: Database> FromSql<DB> for Featured
    where
        i32: FromSql<DB>
{
    fn from_sql(sql: &<DB as Database>::Types) -> Result<Self, Error<DB>>
        where
            Self: Sized
    {
        i32::from_sql(sql).map(Featured::from)
    }
}