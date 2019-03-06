//! Module containing all models releated to Songs

use std::fmt::{Display, Error, Formatter};

#[cfg(feature = "serde_support")]
use serde_derive::{Deserialize, Serialize};

// FIXME: once const_string_new stabilized, turn this into a constant
pub fn SERVER_SIDED_DATA_INCONSISTENCY_ERROR() -> NewgroundsSong {
    NewgroundsSong {
        song_id: 0,
        name: String::new(),
        index_3: 0,
        artist: String::new(),
        filesize: 0f64,
        index_6: None,
        index_7: None,
        index_8: String::new(),
        link: String::new(),
    }
}
/// Struct representing Geometry Dash's main songs.
///
/// This data is not provided by the API and needs to be manually kept up to
/// date
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct MainSong {
    /// The ID of this [`MainSong`]
    pub main_song_id: u8,

    /// The name of this [`MainSong`]
    pub name: &'static str,

    /// The artist of this [`MainSong`]
    pub artist: &'static str,
}

/// Struct representing a Newgrounds song.
///
/// ## GD Internals:
/// The Geometry Dash servers provide a list of the newgrounds songs of the
/// levels in a `getGJLevels` response.
///
/// ### Unused indices:
/// The following indices aren't used by the Geometry Dash servers: `9`
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct NewgroundsSong {
    /// The newgrounds id of this [`NewgroundsSong`]
    ///
    /// ## GD Internals:
    /// This value is provided at index `1`
    pub song_id: u64,

    /// The name of this [`NewgroundsSong`]
    ///
    /// ## GD Internals:
    /// This value is provided at index `2`
    pub name: String,

    pub index_3: u64,

    /// The artist of this [`NewgroundsSong`]
    ///
    /// ## GD Internals:
    /// This value is provided at index `4`
    pub artist: String,

    /// The filesize of this [`NewgroundsSong`], in megabytes
    ///
    /// ## GD Internals:
    /// This value is provided at index `5`
    pub filesize: f64,

    pub index_6: Option<String>,

    // Index 6 has unknown usage
    pub index_7: Option<String>,

    pub index_8: String,

    /// The direct `audio.ngfiles.com` download link for this [`NewgroundsSong`]
    ///
    /// ## GD Internals:
    /// This value is provided at index `10`, and is percent encoded.
    pub link: String,
}

impl MainSong {
    const fn new(main_song_id: u8, name: &'static str, artist: &'static str) -> MainSong {
        MainSong {
            main_song_id,
            name,
            artist,
        }
    }
}

/// All current [`MainSong`]s, as of Geometry Dash 2.1
pub const MAIN_SONGS: [MainSong; 21] = [
    MainSong::new(0, "Stereo Madness", "ForeverBound"),
    MainSong::new(1, "Back on Track", "DJVI"),
    MainSong::new(2, "Polargeist", "Step"),
    MainSong::new(3, "Dry Out", "DJVI"),
    MainSong::new(4, "Base after Base", "DJVI"),
    MainSong::new(5, "Can't Let Go", "DJVI"),
    MainSong::new(6, "Jumper", "Waterflame"),
    MainSong::new(7, "Time Machine", "Waterflame"),
    MainSong::new(8, "Cycles", "DJVI"),
    MainSong::new(9, "xStep", "DJVI"),
    MainSong::new(10, "Clutterfunk", "Waterflame"),
    MainSong::new(11, "Theory of Everything", "DJ-Nate"),
    MainSong::new(12, "Electroman ADventures", "Waterflame"),
    MainSong::new(13, "Clubstep", "DJ-Nate"),
    MainSong::new(14, "Electrodynamix", "DJ-Nate"),
    MainSong::new(15, "Hexagon Force", "Waterflame"),
    MainSong::new(16, "Blast Processing", "Waterflame"),
    MainSong::new(17, "Theory of Everything 2", "DJ-Nate"),
    MainSong::new(18, "Geometrical Dominator", "Waterflame"),
    MainSong::new(19, "Deadlocked", "F-777"),
    MainSong::new(20, "Fingerdash", "MDK"),
];

/// Placeholder value for unknown [`MainSong`]s
///
/// When resolving a main song by its ID, but you pass a wrong ID, or
/// GDCF hasn't updated to include the new song yet, you will receive this object
pub const UNKNOWN: MainSong = MainSong::new(
    0xFF,
    "The song was added after the release of GDCF you're using",
    "Please either update to the newest version, or bug stadust about adding the new songs",
);

impl Display for NewgroundsSong {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "NewgroundsSong({}, {} by {})", self.song_id, self.name, self.artist)
    }
}

impl From<u8> for &'static MainSong {
    fn from(song_id: u8) -> Self {
        MAIN_SONGS.get(song_id as usize).unwrap_or(&UNKNOWN)
    }
}
