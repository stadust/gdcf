use model::RawObject;

pub enum ProcessedResponse {
    One(RawObject),
    Many(Vec<RawObject>),
}