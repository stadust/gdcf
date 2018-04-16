macro_rules! endpoint {
    ($php:expr) => {
        format!("http://www.boomlings.com/database/{}.php", $php).parse().unwrap();
    }
}

macro_rules! prepare_request {
    ($endpoint:expr, $request:expr) => {
        {
            let body = serde_urlencoded::to_string($request).unwrap();
            let mut req = Request::new(Method::Post, endpoint!($endpoint));

            println!("Making request {} to endpoint {}", body, $endpoint);

            req.headers_mut().set(ContentType::form_url_encoded());
            req.headers_mut().set(ContentLength(body.len() as u64));
            req.set_body(body);

            req
        }
    }
}

macro_rules! prepare_future {
    ($future:expr, $parser:expr) => {
        {
            let future = $future
                .map_err(|err| convert_error(err))
                .and_then(|resp| {
                    println!("Got a response {:?}", resp.headers());
                    match resp.status() {
                        StatusCode::InternalServerError => Err(GDError::InternalServerError),
                        StatusCode::NotFound => Err(GDError::ServersDown),
                        _ => Ok(resp)
                    }
                })
                .and_then(|resp| {
                    resp.body()
                        .concat2()
                        .map_err(|err| convert_error(err))
                        .and_then(|body| $parser(str::from_utf8(&body)?))
                });

            Box::new(future)
        }
    }
}

macro_rules! check_resp {
    ($data:expr) => {
        {
            if $data == "-1" {
                return Err(GDError::NoData)
            }
        }
    }
}