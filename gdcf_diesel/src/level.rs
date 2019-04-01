use diesel::ExpressionMethods;
use gdcf_model::level::{Level, Password};

diesel_stuff! {
    level (level_id, Level<u64, u64>) {
        (level_id, Int8, i64, i64),
        (level_data, Binary, Vec<u8>, &'a [u8]),
        (level_password, Nullable<Text>, Option<String>, Option<&'a str>),
        (time_since_upload, Text, String, &'a String),
        (time_since_update, Text, String, &'a String),
        (index_36, Text, String, &'a String)
    }
}

fn values(level: &Level<u64, u64>) -> Values {
    use level::columns::*;

    (
        level_id.eq(level.base.level_id as i64),
        level_data.eq(&level.level_data[..]),
        level_password.eq(match level.password {
            Password::NoCopy => None,
            Password::FreeCopy => Some("1"),
            Password::PasswordCopy(ref password) => Some(password.as_ref()),
        }),
        time_since_upload.eq(&level.time_since_upload),
        time_since_update.eq(&level.time_since_update),
        index_36.eq(&level.index_36),
    )
}
