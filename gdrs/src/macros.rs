macro_rules! endpoint {
    ($php:expr) => {
        format!("http://www.boomlings.com/database/{}.php", $php)
            .parse()
            .unwrap();
    };
}

macro_rules! prepare_future {
    ($future:expr, $parser:expr) => {{
        let future = $future
            .map_err(|err| convert_error(err))
            .and_then(|resp| {
                println!("Got a response {:?}", resp);
                match resp.status() {
                    StatusCode::InternalServerError => Err(GDError::InternalServerError),
                    StatusCode::NotFound => Err(GDError::ServersDown),
                    _ => Ok(resp),
                }
            })
            .and_then(|resp| {
                resp.body()
                    .concat2()
                    .map_err(|err| convert_error(err))
                    .and_then(|body| $parser(str::from_utf8(&body)?))
            });

        Box::new(future)
    }};
}

macro_rules! check_resp {
    ($data:expr) => {{
        if $data == "-1" {
            return Err(GDError::NoData);
        }
    }};
}
