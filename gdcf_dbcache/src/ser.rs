use crate::core::{backend::Database, AsSql};
use gdcf_model::{
    level::{Featured, LevelLength, LevelRating, Password},
    song::MainSong,
    GameVersion,
};

impl<DB: Database> AsSql<DB> for LevelRating
where
    String: AsSql<DB>,
{
    fn as_sql(&self) -> <DB as Database>::Types {
        self.to_string().as_sql()
    }
}

impl<DB: Database> AsSql<DB> for MainSong
where
    u8: AsSql<DB>,
{
    fn as_sql(&self) -> <DB as Database>::Types {
        self.main_song_id.as_sql()
    }
}

impl<DB: Database> AsSql<DB> for GameVersion
where
    u8: AsSql<DB>,
{
    fn as_sql(&self) -> <DB as Database>::Types {
        let v: u8 = (*self).into();
        v.as_sql()
    }
}

impl<DB: Database> AsSql<DB> for LevelLength
where
    String: AsSql<DB>,
{
    fn as_sql(&self) -> <DB as Database>::Types {
        self.to_string().as_sql()
    }
}

impl<DB: Database> AsSql<DB> for Featured
where
    i32: AsSql<DB>,
{
    fn as_sql(&self) -> <DB as Database>::Types {
        let v: i32 = (*self).into();
        v.as_sql()
    }
}

impl<'a, DB: Database + 'a> AsSql<DB> for Password
where
    String: AsSql<DB>,
    Option<String>: AsSql<DB>,
    &'a str: AsSql<DB>,
{
    fn as_sql(&self) -> <DB as Database>::Types {
        match self {
            Password::NoCopy => None.as_sql(),
            Password::FreeCopy => "1".as_sql(),
            Password::PasswordCopy(password) => password.as_sql(),
        }
    }
}
