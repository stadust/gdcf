macro_rules! endpoint {
    ($php:expr) => {
        format!("http://www.boomlings.com/database/{}.php", $php).parse().unwrap();
    };
}

macro_rules! prepare_future {
    ($future:expr, $parser:expr) => {{
        let future = $future
            .map_err(|err| ApiError::Custom(err))
            .and_then(|resp| {
                match resp.status() {
                    StatusCode::INTERNAL_SERVER_ERROR => Err(ApiError::InternalServerError),
                    StatusCode::NOT_FOUND => Err(ApiError::NoData),
                    _ => Ok(resp),
                }
            }).and_then(|resp| {
                resp.into_body().concat2().map_err(|err| ApiError::Custom(err)).and_then(|body| {
                    match str::from_utf8(&body) {
                        Ok(body) => $parser(body),
                        Err(_) => Err(ApiError::UnexpectedFormat),
                    }
                })
            });

        Box::new(future)
    }};
}

macro_rules! check_resp {
    ($data:expr) => {{
        if $data == "-1" {
            return Err(ApiError::NoData)
        }
    }};
}
