use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    web::Data,
    Error, HttpMessage, HttpResponse, HttpResponseBuilder,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use futures_util::future::LocalBoxFuture;

use crate::AppData;

pub struct Auth<const ABORT_IF_NO_USER: bool> {
    pub app_data: Data<AppData>,
}

impl<S, B, const ABORT_IF_NO_USER: bool> Transform<S, ServiceRequest> for Auth<ABORT_IF_NO_USER>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S, ABORT_IF_NO_USER>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            app_data: self.app_data.clone(),
        }))
    }
}

pub struct AuthMiddleware<S, const ABORT_IF_NO_USER: bool> {
    service: Rc<S>,
    app_data: Data<AppData>,
}

fn starts_with_ignore_case(haystack: &str, needle: &str) -> bool {
    let mut h_chars = haystack.chars();
    let mut n_chars = needle.chars();
    // zipping the iterators would stop as soon as
    // the first None would be encountered
    // but there is no way of knowing, which iterator
    // returned None
    // for (hc, nc) in h_chars.zip(n_chars) {
    //     if hc.to_ascii_lowercase() != nc.to_ascii_lowercase() {
    //         return false;
    //     }
    // }
    loop {
        let hc = h_chars.next();
        let nc = n_chars.next();
        // there are no characters in needle anymore
        // and they were equal up to this point -> true
        let nc = match nc {
            None => return true,
            Some(nc) => nc,
        };
        // there still is a character in needle, but
        // not in haystack, so needle must be longer than haystack -> false
        let hc = match hc {
            None => return false,
            Some(hc) => hc,
        };
        if hc.to_ascii_lowercase() != nc.to_ascii_lowercase() {
            return false;
        }
    }
}

fn extract_identifier_and_password(auth_header: &str) -> Option<(String, String)> {
    let trimmed = auth_header.trim_start();
    if !starts_with_ignore_case(trimmed, "basic") {
        return None;
    }
    // since we checked that the first 5 bytes are "basic"
    // we can safely slice
    // trim_start() to remove the whitespace(s) between "basic" and base64 string
    let substr = trimmed[5..].trim_start();
    // decode base64
    let decoded = STANDARD.decode(substr).ok()?;
    let decoded_str = std::str::from_utf8(&decoded).ok()?;
    // split by ':' once
    // first part is identifier
    // second part is password
    decoded_str
        .split_once(':')
        .map(|(id_str, password_str)| (id_str.to_string(), password_str.to_string()))
}

macro_rules! ok_or_log_and_respond_service_internal_server_error {
    ($result: expr, $req: expr) => {
        match $result {
            Ok(res) => res,
            Err(err) => {
                log::error!("Internal server error: {}", err);
                let (request, _) = $req.into_parts();
                let error_response =
                    HttpResponse::InternalServerError().json("internal server error");
                let response = error_response.map_into_right_body();
                return Ok(ServiceResponse::new(request, response));
            }
        }
    };
}

macro_rules! some_or_respond_service_bad_request {
    ($result: expr, $req: expr) => {
        match $result {
            Some(res) => res,
            None => {
                let (request, _) = $req.into_parts();
                let error_response = HttpResponse::BadRequest().json("bad request");
                let response = error_response.map_into_right_body();
                return Ok(ServiceResponse::new(request, response));
            }
        }
    };
}

macro_rules! some_or_respond_service_not_found {
    ($result: expr, $req: expr) => {
        match $result {
            Some(res) => res,
            None => {
                let (request, _) = $req.into_parts();
                let error_response = HttpResponse::NotFound().json("resource not found");
                let response = error_response.map_into_right_body();
                return Ok(ServiceResponse::new(request, response));
            }
        }
    };
}

impl<S, B, const ABORT_IF_NO_USER: bool> Service<ServiceRequest>
    for AuthMiddleware<S, ABORT_IF_NO_USER>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let app_data = self.app_data.clone();

        Box::pin(async move {
            let headers = req.headers();

            let auth_header_option = headers.get("authorization");

            if auth_header_option.is_none() && ABORT_IF_NO_USER {
                let (request, _) = req.into_parts();
                let mut response_builder = HttpResponseBuilder::new(StatusCode::UNAUTHORIZED);
                response_builder.append_header(("www-authenticate", "Basic"));
                let error_response = response_builder.json("supply an authorization header");
                let response = error_response.map_into_right_body();
                return Ok(ServiceResponse::new(request, response));
            }

            if let Some(auth_header) = auth_header_option {
                let auth_header_str =
                    some_or_respond_service_bad_request!(auth_header.to_str().ok(), req);
                let id_and_password_option = extract_identifier_and_password(auth_header_str);
                let (id, password) =
                    some_or_respond_service_bad_request!(id_and_password_option, req);

                let user_result =
                    sqlx::query!("select id, password from users where username = $1", &id)
                        .fetch_optional(&app_data.pool)
                        .await;
                let user_option =
                    ok_or_log_and_respond_service_internal_server_error!(user_result, req);
                let user = some_or_respond_service_not_found!(user_option, req);

                let verification_result = bcrypt::verify(password, &user.password);
                let verified =
                    ok_or_log_and_respond_service_internal_server_error!(verification_result, req);
                if verified {
                    req.extensions_mut().insert(user.id);
                }
            }
            let fut = service.call(req);

            // regular endpoint
            let res = fut.await?.map_into_left_body();
            Ok(res)
        })
    }
}
