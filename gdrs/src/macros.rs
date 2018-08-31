macro_rules! endpoint {
    ($php:expr) => {
        format!("http://www.boomlings.com/database/{}.php", $php).parse().unwrap();
    };
}

macro_rules! api_call {
    ($api: ident, $request_type: ident, $endpoint: expr, $parser: expr) => {
        fn $api(&self, req: $request_type) -> ApiFuture<Self::Err> {
            let action = ApiRequestAction {
                client: self.client.clone(),
                endpoint: $endpoint,
                request: Req::$request_type(req),
                parser: $parser
            };

            let retry = ExponentialBackoff::from_millis(10).map(jitter).take(5);

            Box::new(RetryIf::spawn(retry, action, ApiRetryCondition).map_err(|err| {
                match err {
                    RetryError::OperationError(e) => e,
                    _ => unimplemented!(),
                }
            }))
        }
    }
}

macro_rules! check_resp {
    ($data:expr) => {{
        if $data == "-1" {
            return Err(ApiError::NoData)
        }
    }};
}
