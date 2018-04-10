use model::GameVersion;
use serde::Serializer;

pub(super) fn game_version<S>(version: &GameVersion, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    serializer.collect_str(&version.to_string())
}

pub(super) fn bool_to_int<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
{
    serializer.serialize_u8(*value as u8)
}