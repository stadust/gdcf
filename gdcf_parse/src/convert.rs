use crate::util::{b64_decode_string, xor_decrypt};
use gdcf_model::{
    level::{Featured, LevelLength, Password},
    GameVersion,
};
use std::{borrow::Borrow, str::FromStr};

pub trait RobtopConvert<T: Borrow<BT>, BT: ?Sized>: Sized {
    fn robtop_from(s: &BT) -> Result<Self, String>;
    fn robtop_into(self) -> T;

    fn can_omit(&self) -> bool {
        false
    }
}

impl RobtopConvert<String, str> for bool {
    fn robtop_from(s: &str) -> Result<Self, String> {
        match s {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err("Not '0' or '1'".to_owned()),
        }
    }

    fn robtop_into(self) -> String {
        match self {
            true => "1",
            false => "0",
        }
        .to_string()
    }

    fn can_omit(&self) -> bool {
        !*self
    }
}

macro_rules! delegate_to_from_str {
    ($($t: ty),*) => {
        $(
            impl RobtopConvert<String, str> for $t {
                fn robtop_from(s: &str ) -> Result<Self, String> {
                    s.parse().map_err(|e: <Self as FromStr>::Err| e.to_string())
                }

                fn robtop_into(self) -> String {
                    self.to_string()
                }

                fn can_omit(&self) -> bool {
                    *self == Self::default()
                }
            }
        )*
    };
}

delegate_to_from_str!(i8, u8, i16, u16, i32, u32, i64, u64, usize, isize, f32, f64);

impl RobtopConvert<String, str> for String {
    fn robtop_from(s: &str) -> Result<Self, String> {
        Ok(s.to_string())
    }

    fn robtop_into(self) -> String {
        self
    }

    fn can_omit(&self) -> bool {
        self.is_empty()
    }
}

macro_rules! delegate_to_from_str_no_omit {
    ($($t: ty),*) => {
        $(
            impl RobtopConvert<String, str> for $t {
                fn robtop_from(s: &str ) -> Result<Self, String> {
                    s.parse().map_err(|e: <Self as FromStr>::Err| e.to_string())
                }

                fn robtop_into(self) -> String {
                    self.to_string()
                }
            }
        )*
    };
}

delegate_to_from_str_no_omit!(GameVersion);

impl RobtopConvert<String, str> for Featured {
    fn robtop_from(s: &str) -> Result<Self, String> {
        match s {
            "-1" => Ok(Featured::Unfeatured),
            "0" => Ok(Featured::NotFeatured),
            other => other.parse().map(Featured::Featured).map_err(|e| e.to_string()),
        }
    }

    fn robtop_into(self) -> String {
        match self {
            Featured::Unfeatured => "-1".to_string(),
            Featured::NotFeatured => "0".to_string(),
            Featured::Featured(value) => value.to_string(),
        }
    }
}

impl RobtopConvert<String, str> for LevelLength {
    fn robtop_from(s: &str) -> Result<Self, String> {
        Ok(match s {
            "0" => LevelLength::Tiny,
            "1" => LevelLength::Short,
            "2" => LevelLength::Medium,
            "3" => LevelLength::Long,
            "4" => LevelLength::ExtraLong,
            _ => LevelLength::Unknown,
        })
    }

    fn robtop_into(self) -> String {
        match self {
            LevelLength::Tiny => "0",
            LevelLength::Short => "1",
            LevelLength::Medium => "2",
            LevelLength::Long => "3",
            LevelLength::ExtraLong => "4",
            _ => "",
        }
        .to_string()
    }
}

impl RobtopConvert<String, str> for Password {
    /// Attempts to parse the given `str` into a [`Password`]
    ///
    /// # Errors
    /// If the given string isn't `"0"` and also isn't valid URL-safe base64, a
    /// [`DecodeError`] is returned
    fn robtop_from(encrypted: &str) -> Result<Self, String> {
        match encrypted {
            "0" => Ok(Password::NoCopy),
            pass => {
                let decoded = b64_decode_string(pass).map_err(|e| e.to_string())?;
                let mut decrypted = xor_decrypt(&decoded, "26364");

                if decrypted.len() == 1 {
                    Ok(Password::FreeCopy)
                } else {
                    decrypted.remove(0);
                    Ok(Password::PasswordCopy(decrypted))
                }
            },
        }
    }

    fn robtop_into(self) -> String {
        let encrypted = match self {
            Password::NoCopy => return "0".to_string(),
            Password::FreeCopy => xor_decrypt("1", "26364"),
            Password::PasswordCopy(pw) => xor_decrypt(&format!("0{}", pw), "26364"),
        };

        base64::encode_config(&encrypted, base64::URL_SAFE)
    }
}
