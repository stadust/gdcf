use crate::util::{self, b64_decode_string, xor_decrypt};
use gdcf_model::{level::{data::portal::Speed, Featured, LevelLength, Password}, user::ModLevel, GameVersion, GameMode};
use percent_encoding::{percent_decode, percent_encode, SIMPLE_ENCODE_SET};
use std::str::FromStr;
use gdcf_model::user::Color;

pub trait RobtopInto<Conv, T> {
    fn robtop_into(self) -> T;

    fn can_omit(&self) -> bool {
        false
    }
}

pub trait RobtopFrom<For, T>: Sized {
    fn robtop_from(t: T) -> Result<For, String>;
}

pub trait RobtopFromInfallible<For, T> {
    fn robtop_from_infallible(s: T) -> For;
}

impl RobtopInto<bool, String> for bool {
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

impl RobtopFrom<bool, &str> for bool {
    fn robtop_from(s: &str) -> Result<bool, String> {
        match s {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err("Not '0' or '1'".to_owned()),
        }
    }
}

// because robtop wants me to suffer an early death
pub(crate) struct TwoBool;

impl RobtopInto<TwoBool, String> for bool {
    fn robtop_into(self) -> String {
        match self {
            true => "2",
            false => "0",
        }
            .to_string()
    }

    fn can_omit(&self) -> bool {
        !*self
    }
}

impl RobtopFrom<bool, &str> for TwoBool {
    fn robtop_from(s: &str) -> Result<bool, String> {
        match s {
            "0" => Ok(false),
            "2" => Ok(true),
            _ => Err("Not '0' or '2'".to_owned()),
        }
    }
}

macro_rules! delegate_to_from_str {
    ($($t: ident),*) => {
        $(
            impl RobtopInto<$t, String> for $t {
                fn robtop_into(self) -> String {
                    self.to_string()
                }

                fn can_omit(&self) -> bool {
                    *self == $t::default()
                }
            }

            impl RobtopFrom<$t, &str> for $t {
                fn robtop_from(s: &str ) -> Result<$t, String> {
                    s.parse().map_err(|e: <Self as FromStr>::Err| e.to_string())
                }
            }
        )*
    };
}

delegate_to_from_str!(i8, u8, i16, u16, i32, u32, i64, u64, usize, isize, f32, f64);

macro_rules! delegate_into_num {
    ($t: ident[$num: ty]) => {
        impl RobtopInto<$t, String> for $t {
            fn robtop_into(self) -> String {
                let intermediate: $num = self.into();
                intermediate.to_string()
            }
        }
    };
}

impl RobtopInto<String, String> for String {
    fn robtop_into(self) -> String {
        self
    }

    fn can_omit(&self) -> bool {
        self.is_empty()
    }
}

impl RobtopFrom<String, &str> for String {
    fn robtop_from(s: &str) -> Result<String, String> {
        Ok(s.to_string())
    }
}

impl RobtopFrom<GameVersion, &str> for GameVersion {
    fn robtop_from(s: &str) -> Result<GameVersion, String> {
        s.parse().map(u8::into).map_err(|e| e.to_string())
    }
}
impl RobtopInto<GameVersion, String> for GameVersion {
    fn robtop_into(self) -> String {
        match self {
            GameVersion::Unknown => String::from("10"),
            GameVersion::Version { minor, major } => (minor + 10 * major).to_string(),
        }
    }
}

impl RobtopFrom<Featured, &str> for Featured {
    fn robtop_from(s: &str) -> Result<Featured, String> {
        match s {
            "-1" => Ok(Featured::Unfeatured),
            "0" => Ok(Featured::NotFeatured),
            other => other.parse().map(Featured::Featured).map_err(|e| e.to_string()),
        }
    }
}

impl RobtopInto<Featured, String> for Featured {
    fn robtop_into(self) -> String {
        match self {
            Featured::Unfeatured => "-1".to_string(),
            Featured::NotFeatured => "0".to_string(),
            Featured::Featured(value) => value.to_string(),
        }
    }
}

impl RobtopFrom<LevelLength, &str> for LevelLength {
    fn robtop_from(s: &str) -> Result<LevelLength, String> {
        Ok(match s {
            "0" => LevelLength::Tiny,
            "1" => LevelLength::Short,
            "2" => LevelLength::Medium,
            "3" => LevelLength::Long,
            "4" => LevelLength::ExtraLong,
            s => LevelLength::Unknown(i32::robtop_from(s)?),
        })
    }
}

impl RobtopInto<LevelLength, String> for LevelLength {
    fn robtop_into(self) -> String {
        match self {
            LevelLength::Tiny => "0".to_string(),
            LevelLength::Short => "1".to_string(),
            LevelLength::Medium => "2".to_string(),
            LevelLength::Long => "3".to_string(),
            LevelLength::ExtraLong => "4".to_string(),
            LevelLength::Unknown(value) => value.to_string(),
        }

    }
}

impl RobtopFrom<Password, &str> for Password {
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
}

impl RobtopInto<Password, String> for Password {
    fn robtop_into(self) -> String {
        let encrypted = match self {
            Password::NoCopy => return "0".to_string(),
            Password::FreeCopy => xor_decrypt("1", "26364"),
            Password::PasswordCopy(pw) => xor_decrypt(&format!("0{}", pw), "26364"),
        };

        base64::encode_config(&encrypted, base64::URL_SAFE)
    }
}

impl RobtopFrom<Speed, &str> for Speed {
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
}

impl RobtopInto<Speed, String> for Speed {
    fn robtop_into(self) -> String {
        match self {
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

impl RobtopInto<ModLevel, String> for ModLevel {
    fn robtop_into(self) -> String {
        match self {
            ModLevel::None => "0".to_string(),
            ModLevel::Normal => "1".to_string(),
            ModLevel::Elder => "2".to_string(),
            ModLevel::Unknown(inner) => inner.to_string(),
        }
    }
}

impl RobtopFrom<ModLevel, &str> for ModLevel {
    fn robtop_from(t: &str) -> Result<ModLevel, String> {
        Ok(match t {
            "0" => ModLevel::None,
            "1" => ModLevel::Normal,
            "2" => ModLevel::Elder,
            t => ModLevel::Unknown(u8::robtop_from(t)?),
        })
    }
}

delegate_into_num!(GameMode[u8]);

impl RobtopFrom<GameMode ,&str> for GameMode {
    fn robtop_from(t: &str) -> Result<GameMode, String> {
        Ok(match t {
            "0" => GameMode::Cube,
            "1" => GameMode::Ship,
            "2" => GameMode::Ball,
            "3" => GameMode::Ufo,
            "4" => GameMode::Wave,
            "5" => GameMode::Robot,
            "6" => GameMode::Spider,
            i => GameMode::Unknown(u8::robtop_from(i)?)
        })
    }
}

impl RobtopInto<Color, String> for Color {
    fn robtop_into(self) -> String {
        match self {
            Color::Known(125, 255, 0) => "0".to_string(),
            Color::Known(0, 255, 0) => "1".to_string(),
            Color::Known(0, 255, 125) => "2".to_string(),
            Color::Known(0, 255, 255) => "3".to_string(),
            Color::Known(0, 200, 255) => "16".to_string(),
            Color::Known(0, 125, 255) => "4".to_string(),
            Color::Known(0, 0, 255) => "5".to_string(),
            Color::Known(125, 0, 255) => "6".to_string(),
            Color::Known(185, 0, 255) => "13".to_string(),
            Color::Known(255, 0, 255) => "7".to_string(),
            Color::Known(255, 0, 125) => "8".to_string(),
            Color::Known(255, 0, 0) => "9".to_string(),
            Color::Known(255, 75, 0) => "29".to_string(),
            Color::Known(255, 125, 0) => "10".to_string(),
            Color::Known(255, 185, 0) => "14".to_string(),
            Color::Known(255, 255, 0) => "11".to_string(),
            Color::Known(255, 255, 255) => "12".to_string(),
            Color::Known(175, 175, 175) => "17".to_string(),
            Color::Known(80, 80, 80) => "18".to_string(),
            Color::Known(0, 0, 0) => "15".to_string(),
            Color::Known(125, 125, 0) => "27".to_string(),
            Color::Known(100, 150, 0) => "32".to_string(),
            Color::Known(75, 175, 0) => "28".to_string(),
            Color::Known(0, 150, 0) => "38".to_string(),
            Color::Known(0, 175, 75) => "20".to_string(),
            Color::Known(0, 150, 100) => "33".to_string(),
            Color::Known(0, 125, 125) => "21".to_string(),
            Color::Known(0, 100, 150) => "34".to_string(),
            Color::Known(0, 75, 175) => "22".to_string(),
            Color::Known(0, 0, 150) => "39".to_string(),
            Color::Known(75, 0, 175) => "23".to_string(),
            Color::Known(100, 0, 150) => "35".to_string(),
            Color::Known(125, 0, 125) => "24".to_string(),
            Color::Known(150, 0, 100) => "36".to_string(),
            Color::Known(175, 0, 75) => "25".to_string(),
            Color::Known(150, 0, 0) => "37".to_string(),
            Color::Known(150, 50, 0) => "30".to_string(),
            Color::Known(175, 75, 0) => "26".to_string(),
            Color::Known(150, 100, 0) => "31".to_string(),
            Color::Known(255, 255, 125) => "19".to_string(),
            Color::Known(125, 255, 175) => "40".to_string(),
            Color::Known(125, 125, 255) => "41".to_string(),
            Color::Unknown(idx) => idx.to_string(),
            _ => "Non GD Color (This is an error on your end. You fucked up)".to_string(),
        }
    }
}

impl RobtopFrom<Color, &str> for Color {
    fn robtop_from(t: &str) -> Result<Color, String> {
        Ok(Color::from(u8::robtop_from(t)?))
    }
}

impl<D: Default + PartialEq, T> RobtopFrom<Option<D>, T> for Option<D>
where
    D: RobtopFrom<D, T>,
{
    fn robtop_from(s: T) -> Result<Option<D>, String> {
        D::robtop_from(s).map(|inner| if inner == D::default() { None } else { Some(inner) })
    }
}

impl<D: Default, T> RobtopInto<Option<D>, T> for Option<D>
where
    D: RobtopInto<D, T>,
{
    fn robtop_into(self) -> T {
        D::robtop_into(match self {
            None => D::default(),
            Some(d) => d,
        })
    }
}

pub struct Base64BytesConverter;

impl RobtopFrom<Vec<u8>, &str> for Base64BytesConverter {
    fn robtop_from(s: &str) -> Result<Vec<u8>, String> {
        base64::decode_config(s, base64::URL_SAFE).map_err(|e| e.to_string())
    }
}

impl RobtopInto<Base64BytesConverter, String> for Vec<u8> {
    fn robtop_into(self) -> String {
        base64::encode_config(&self, base64::URL_SAFE)
    }
}

pub struct Base64Converter;

impl RobtopFromInfallible<Option<String>, &str> for Base64Converter {
    fn robtop_from_infallible(s: &str) -> Option<String> {
        util::b64_decode_string(s).ok()
    }
}

impl RobtopInto<Base64Converter, String> for Option<String> {
    fn robtop_into(self) -> String {
        match self {
            Some(ref desc) => base64::encode_config(desc, base64::URL_SAFE),
            None => String::new(),
        }
    }
}

impl RobtopFrom<String, &str> for Base64Converter {
    fn robtop_from(s: &str) -> Result<String, String> {
        util::b64_decode_string(s).map_err(|e| e.to_string())
    }
}

impl RobtopInto<Base64Converter, String> for String {
    fn robtop_into(self) -> String {
        base64::encode_config(&self, base64::URL_SAFE)
    }
}

pub struct UrlConverter;

impl RobtopFrom<String, &str> for UrlConverter {
    fn robtop_from(s: &str) -> Result<String, String> {
        let utf8_cow = percent_decode(s.as_bytes()).decode_utf8().map_err(|e| e.to_string())?;

        Ok(utf8_cow.to_string())
    }
}

impl RobtopInto<UrlConverter, String> for String {
    fn robtop_into(self) -> String {
        percent_encode(self.as_bytes(), SIMPLE_ENCODE_SET).to_string()
    }
}

pub struct YoutubeConverter;
pub struct TwitterConverter;
pub struct TwitchConverter;

impl RobtopInto<YoutubeConverter, String> for Option<String> {
    fn robtop_into(self) -> String {
        self.map(|url| url.rsplit('/').next().unwrap().to_string()).unwrap_or_default()
    }
}

impl RobtopInto<TwitterConverter, String> for Option<String> {
    fn robtop_into(self) -> String {
        self.map(|url| url.rsplit('/').next().unwrap().to_string()).unwrap_or_default()
    }
}

impl RobtopInto<TwitchConverter, String> for Option<String> {
    fn robtop_into(self) -> String {
        self.map(|url| url.rsplit('/').next().unwrap().to_string()).unwrap_or_default()
    }
}

impl RobtopFromInfallible<Option<String>, &str> for YoutubeConverter {
    fn robtop_from_infallible(value: &str) -> Option<String> {
        if value.is_empty() {
            None
        } else {
            Some(format!("https://www.youtube.com/channel/{}", value))
        }
    }
}

impl RobtopFromInfallible<Option<String>, &str> for TwitterConverter {
    fn robtop_from_infallible(value: &str) -> Option<String> {
        if value.is_empty() {
            None
        } else {
            Some(format!("https://www.twitter.com/{}", value))
        }
    }
}

impl RobtopFromInfallible<Option<String>, &str> for TwitchConverter {
    fn robtop_from_infallible(value: &str) -> Option<String> {
        if value.is_empty() {
            None
        } else {
            Some(format!("https://www.twitch.tv/{}", value))
        }
    }
}
