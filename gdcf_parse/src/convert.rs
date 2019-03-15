use crate::util::{self, b64_decode_string, xor_decrypt};
use gdcf_model::{
    level::{data::portal::Speed, Featured, LevelLength, Password},
    GameVersion,
};
use percent_encoding::{percent_decode, percent_encode, SIMPLE_ENCODE_SET};
use std::{borrow::Borrow, str::FromStr};

pub trait RobtopConvert<For, T: Borrow<BT>, BT: ?Sized>: Sized {
    fn robtop_from(s: &BT) -> Result<For, String>;
    fn robtop_into(f: For) -> T;

    fn can_omit(f: &For) -> bool {
        false
    }
}

impl RobtopConvert<bool, String, str> for bool {
    fn robtop_from(s: &str) -> Result<bool, String> {
        match s {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err("Not '0' or '1'".to_owned()),
        }
    }

    fn robtop_into(b: bool) -> String {
        match b {
            true => "1",
            false => "0",
        }
        .to_string()
    }

    fn can_omit(b: &bool) -> bool {
        !*b
    }
}

macro_rules! delegate_to_from_str {
    ($($t: ident),*) => {
        $(
            impl RobtopConvert<$t, String, str> for $t {
                fn robtop_from(s: &str ) -> Result<$t, String> {
                    s.parse().map_err(|e: <Self as FromStr>::Err| e.to_string())
                }

                fn robtop_into(t: $t) -> String {
                    t.to_string()
                }

                fn can_omit(t: &$t) -> bool {
                    *t == $t::default()
                }
            }
        )*
    };
}

delegate_to_from_str!(i8, u8, i16, u16, i32, u32, i64, u64, usize, isize, f32, f64);

impl RobtopConvert<String, String, str> for String {
    fn robtop_from(s: &str) -> Result<String, String> {
        Ok(s.to_string())
    }

    fn robtop_into(s: String) -> String {
        s
    }

    fn can_omit(s: &String) -> bool {
        s.is_empty()
    }
}

macro_rules! delegate_to_from_str_no_omit {
    ($($t: ty),*) => {
        $(
            impl RobtopConvert<$t, String, str> for $t {
                fn robtop_from(s: &str ) -> Result<$t, String> {
                    s.parse().map_err(|e: <Self as FromStr>::Err| e.to_string())
                }

                fn robtop_into(t: $t) -> String {
                    t.to_string()
                }
            }
        )*
    };
}

delegate_to_from_str_no_omit!(GameVersion);

impl RobtopConvert<Featured, String, str> for Featured {
    fn robtop_from(s: &str) -> Result<Featured, String> {
        match s {
            "-1" => Ok(Featured::Unfeatured),
            "0" => Ok(Featured::NotFeatured),
            other => other.parse().map(Featured::Featured).map_err(|e| e.to_string()),
        }
    }

    fn robtop_into(f: Featured) -> String {
        match f {
            Featured::Unfeatured => "-1".to_string(),
            Featured::NotFeatured => "0".to_string(),
            Featured::Featured(value) => value.to_string(),
        }
    }
}

impl RobtopConvert<LevelLength, String, str> for LevelLength {
    fn robtop_from(s: &str) -> Result<LevelLength, String> {
        Ok(match s {
            "0" => LevelLength::Tiny,
            "1" => LevelLength::Short,
            "2" => LevelLength::Medium,
            "3" => LevelLength::Long,
            "4" => LevelLength::ExtraLong,
            _ => LevelLength::Unknown,
        })
    }

    fn robtop_into(length: LevelLength) -> String {
        match length {
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

impl RobtopConvert<Password, String, str> for Password {
    /// Attempts to parse the given `str` into a [`Password`]
    ///
    /// # Errors
    /// If the given string isn't `"0"` and also isn't valid URL-safe base64, a
    /// [`DecodeError`] is returned
    fn robtop_from(encrypted: &str) -> Result<Password, String> {
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

    fn robtop_into(pass: Password) -> String {
        let encrypted = match pass {
            Password::NoCopy => return "0".to_string(),
            Password::FreeCopy => xor_decrypt("1", "26364"),
            Password::PasswordCopy(pw) => xor_decrypt(&format!("0{}", pw), "26364"),
        };

        base64::encode_config(&encrypted, base64::URL_SAFE)
    }
}

impl RobtopConvert<Speed, String, str> for Speed {
    fn robtop_from(speed: &str) -> Result<Speed, String> {
        match speed {
            "0" => Ok(Speed::Slow),
            "1" => Ok(Speed::Normal),
            "2" => Ok(Speed::Medium),
            "3" => Ok(Speed::Fast),
            "4" => Ok(Speed::VeryFast),
            _ => Err("Unknown speed value".to_string()),
        }
    }

    fn robtop_into(speed: Speed) -> String {
        match speed {
            Speed::Slow => "0",
            Speed::Normal => "1",
            Speed::Medium => "2",
            Speed::Fast => "3",
            Speed::VeryFast => "4",
            Speed::Invalid => "",
        }
        .to_string()
    }
}

impl<D: Default + PartialEq, BT: ?Sized, T: Borrow<BT>> RobtopConvert<Option<D>, T, BT> for Option<D>
where
    D: RobtopConvert<D, T, BT>,
{
    fn robtop_from(s: &BT) -> Result<Option<D>, String> {
        D::robtop_from(s).map(|inner| if inner == D::default() { None } else { Some(inner) })
    }

    fn robtop_into(option: Option<D>) -> T {
        D::robtop_into(match option {
            None => D::default(),
            Some(d) => d,
        })
    }
}

pub trait InfallibleRobtopConvert<For, T: Borrow<BT>, BT: ?Sized> {
    fn robtop_from_infallible(s: &BT) -> For;
    fn robtop_into_infallible(f: For) -> T;

    fn can_omit(&self) -> bool {
        false
    }
}

pub struct Base64BytesConverter;

impl RobtopConvert<Vec<u8>, String, str> for Base64BytesConverter {
    fn robtop_from(s: &str) -> Result<Vec<u8>, String> {
        base64::decode_config(s, base64::URL_SAFE).map_err(|e| e.to_string())
    }

    fn robtop_into(f: Vec<u8>) -> String {
        base64::encode_config(&f, base64::URL_SAFE)
    }
}

pub struct Base64Converter;

impl InfallibleRobtopConvert<Option<String>, String, str> for Base64Converter {
    fn robtop_from_infallible(s: &str) -> Option<String> {
        util::b64_decode_string(s).ok()
    }

    fn robtop_into_infallible(f: Option<String>) -> String {
        match f {
            Some(desc) => base64::encode_config(&desc, base64::URL_SAFE),
            None => String::new(),
        }
    }
}

impl RobtopConvert<String, String, str> for Base64Converter {
    fn robtop_from(s: &str) -> Result<String, String> {
        util::b64_decode_string(s).map_err(|e| e.to_string())
    }

    fn robtop_into(f: String) -> String {
        base64::encode_config(&f, base64::URL_SAFE)
    }
}

pub struct UrlConverter;

impl RobtopConvert<String, String, str> for UrlConverter {
    fn robtop_from(s: &str) -> Result<String, String> {
        let utf8_cow = percent_decode(s.as_bytes()).decode_utf8().map_err(|e| e.to_string())?;

        Ok(utf8_cow.to_string())
    }

    fn robtop_into(f: String) -> String {
        percent_encode(f.as_bytes(), SIMPLE_ENCODE_SET).to_string()
    }
}

pub struct YoutubeConverter;
pub struct TwitterConverter;
pub struct TwitchConverter;

impl InfallibleRobtopConvert<String, String, str> for YoutubeConverter {
    fn robtop_from_infallible(s: &str) -> String {
        unimplemented!()
    }

    fn robtop_into_infallible(f: String) -> String {
        unimplemented!()
    }
}

impl InfallibleRobtopConvert<String, String, str> for TwitterConverter {
    fn robtop_from_infallible(s: &str) -> String {
        unimplemented!()
    }

    fn robtop_into_infallible(f: String) -> String {
        unimplemented!()
    }
}

impl InfallibleRobtopConvert<String, String, str> for TwitchConverter {
    fn robtop_from_infallible(s: &str) -> String {
        unimplemented!()
    }

    fn robtop_into_infallible(f: String) -> String {
        unimplemented!()
    }
}
