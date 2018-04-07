use model::level::GameVersion;

pub mod level;

pub trait Request {
    fn form_data(&self) -> Vec<(&str, String)>;
}

pub trait Paginatable: Request {
    fn next(&self) -> Self;
}

pub struct BaseRequest {
    game_version: GameVersion,
    binary_version: GameVersion,
    secret: String,
}

impl BaseRequest {
    pub fn new(game_version: GameVersion, binary_version: GameVersion, secret: String) -> BaseRequest {
        BaseRequest {
            game_version,
            binary_version,
            secret,
        }
    }

    pub fn gd_21() -> BaseRequest {
        BaseRequest::new(
            GameVersion::Version { major: 2, minor: 1 },
            GameVersion::Version { major: 3, minor: 1 },
            String::from("sdjklf"), // TODO: get the proper stuff here
        )
    }
}

impl Default for BaseRequest {
    fn default() -> Self {
        BaseRequest::gd_21()
    }
}

impl Request for BaseRequest {
    fn form_data(&self) -> Vec<(&str, String)> {
        vec![
            ("gameVersion", self.game_version.to_string()),
            ("binaryVersion", self.binary_version.to_string()),
            ("secret", self.secret.clone())
        ]
    }
}