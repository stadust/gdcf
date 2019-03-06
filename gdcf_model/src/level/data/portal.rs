use crate::level::data::ids;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Slow,
    Normal,
    Medium,
    Fast,
    VeryFast,
    Invalid,
}

impl Default for Speed {
    fn default() -> Speed {
        Speed::Normal
    }
}

impl Into<f32> for Speed {
    fn into(self) -> f32 {
        match self {
            Speed::Invalid => 0.0,
            Speed::Slow => 251.16,
            Speed::Normal => 311.58,
            Speed::Medium => 387.42,
            Speed::Fast => 468.0,
            Speed::VeryFast => 576.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalType {
    Nonsense,
    Speed(Speed),
    // .. all the other portals ..
}

impl PortalType {
    pub fn from_id(id: u16) -> PortalType {
        match id {
            ids::SLOW_PORTAL => PortalType::Speed(Speed::Slow),
            ids::NORMAL_PORTAL => PortalType::Speed(Speed::Normal),
            ids::MEDIUM_PORTAL => PortalType::Speed(Speed::Medium),
            ids::FAST_PORTAL => PortalType::Speed(Speed::Fast),
            ids::VERY_FAST_PORTAL => PortalType::Speed(Speed::VeryFast),
            _ => PortalType::Nonsense,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalData {
    pub checked: bool,
    pub portal_type: PortalType,
}

pub fn get_seconds_from_x_pos(pos: f32, start_speed: Speed, portals: &[(f32, Speed)]) -> f32 {
    let mut speed: f32 = start_speed.into();

    if portals.is_empty() {
        return pos / speed
    }

    let mut last_obj_pos = 0.0;
    let mut total_time = 0.0;

    for (x, portal_speed) in portals {
        // distance between last portal and this one
        let current_segment = x - last_obj_pos;

        // break if we're past the position we want to calculate the position to
        if pos <= current_segment {
            break
        }

        // Calculate time spent in this segment and add to total time
        total_time += current_segment / speed;

        speed = (*portal_speed).into();

        last_obj_pos = *x;
    }

    // add the time spent between end and last portal to total time and return
    (pos - last_obj_pos) / speed + total_time
}
