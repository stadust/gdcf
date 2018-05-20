use core::AsSql;
use core::backend::Database;
use gdcf::model::GameVersion;
use gdcf::model::LevelLength;
use gdcf::model::LevelRating;
use gdcf::model::MainSong;
use gdcf::model::level::Featured;

impl<DB: Database> AsSql<DB> for LevelRating
    where
        String: AsSql<DB>
{
    fn as_sql(&self) -> <DB as Database>::Types {
        self.to_string().as_sql()
    }

    fn as_sql_string(&self) -> String {
        self.to_string().as_sql_string()
    }
}

impl<DB: Database> AsSql<DB> for MainSong
    where
        u8: AsSql<DB>
{
    fn as_sql(&self) -> <DB as Database>::Types {
        self.main_song_id.as_sql()
    }

    fn as_sql_string(&self) -> String {
        self.main_song_id.as_sql_string()
    }
}

impl<DB: Database> AsSql<DB> for GameVersion
    where
        u8: AsSql<DB>
{
    fn as_sql(&self) -> <DB as Database>::Types {
        let v: u8 = (*self).into();
        v.as_sql()
    }

    fn as_sql_string(&self) -> String {
        let v: u8 = (*self).into();
        v.as_sql_string()
    }
}

impl<DB: Database> AsSql<DB> for LevelLength
    where
        String: AsSql<DB>
{
    fn as_sql(&self) -> <DB as Database>::Types {
        (*self).to_string().as_sql()
    }

    fn as_sql_string(&self) -> String {
        self.to_string().as_sql_string()
    }
}

impl<DB: Database> AsSql<DB> for Featured
    where
        i32: AsSql<DB>
{
    fn as_sql(&self) -> <DB as Database>::Types {
        let v: i32 = (*self).into();
        v.as_sql()
    }

    fn as_sql_string(&self) -> String {
        let v: i32 = (*self).into();
        v.as_sql_string()
    }
}