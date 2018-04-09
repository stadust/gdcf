use gdcf::model::GDObject;
use gdcf::api::client::GDError;

pub fn level(body: &str) -> Result<GDObject, GDError> {
    check_resp!(body);

    //let sections = body.split("#").collect();
    //let data = Vec::new();

    println!("{}", body);

    Err(GDError::Unspecified)
}