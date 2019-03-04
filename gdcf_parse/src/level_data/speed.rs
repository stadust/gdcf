#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Slow,
    Normal,
    Medium,
    Fast,
    VeryFast,
    Invalid,
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

pub fn get_seconds_from_x_pos(pos: f32, start_speed: Speed, portals: &[(f32, Speed)]) -> f32 {
    let mut speed: f32 = start_speed.into();

    if portals.is_empty() {
        return pos / speed
    }

    let mut last_obj_pos = 0.0;

    let mut all_segments = 0.0;
    let mut last_big_segment = 0.0;

    for (x, portal_speed) in portals {
        let current_segment = x - last_obj_pos;

        if pos >= current_segment {
            break
        }

        all_segments += current_segment / speed;
        last_big_segment = current_segment;

        speed = (*portal_speed).into();

        last_obj_pos = *x;
    }

    (pos - last_big_segment) / speed + all_segments
}
