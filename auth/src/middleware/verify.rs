use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    dev::{Service, ServiceRequest, Transform},
    Error, HttpMessage,
};
use error::Error as AppError;
use futures_util::future::{ok, LocalBoxFuture, Ready};

use crate::data::authenticated::Authenticated;

#[derive(PartialEq, Debug, Clone)]
pub enum CsrfVerify {
    Header(String),
    Query(String),
    Body(String),
}

/// Verify middleware
///
/// This middleware will verify if the request has a valid user and session.
/// It can also be instantiated in a way so it verifies the CSRF token on the request.
///
/// For csrf verification the request needs to have valid user and session which means
/// if the authentication has been done via the signature method the csrf token will be ignored
/// even if its send in the request.
pub struct Verify {
    pub(crate) csrf_verify: Option<CsrfVerify>,
}

impl Verify {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Verify {
        Verify { csrf_verify: None }
    }

    pub fn new_with(csrf_verify: CsrfVerify) -> Verify {
        Verify {
            csrf_verify: Some(csrf_verify),
        }
    }
    pub fn csrf_header_name(name: String) -> Self {
        Self::new_with(CsrfVerify::Header(name))
    }

    pub fn csrf_header_default() -> Self {
        Self::new_with(CsrfVerify::Header("X-CSRF-TOKEN".to_string()))
    }

    pub fn csrf_query_name(name: String) -> Self {
        Self::new_with(CsrfVerify::Query(name))
    }

    pub fn csrf_query_default() -> Self {
        Self::new_with(CsrfVerify::Query("__csrf".to_string()))
    }

    pub fn csrf_body_name(name: String) -> Self {
        Self::new_with(CsrfVerify::Body(name))
    }

    pub fn csrf_body_default() -> Self {
        Self::new_with(CsrfVerify::Body("__csrf".to_string()))
    }
}
impl<S> Transform<S, ServiceRequest> for Verify
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = VerifyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(VerifyMiddleware {
            service,
            csrf_verify: self.csrf_verify.clone(),
        })
    }
}

pub struct VerifyMiddleware<S> {
    service: S,
    csrf_verify: Option<CsrfVerify>,
}

impl<S> VerifyMiddleware<S> {
    fn should_verify_csrf(&self) -> bool {
        self.csrf_verify.is_some()
    }

    fn extract_csrf(&self, req: &ServiceRequest) -> Option<String> {
        if let Some(csrf_verify) = &self.csrf_verify {
            match csrf_verify {
                CsrfVerify::Header(name) => {
                    let header = req.headers().get(name)?;
                    let csrf = header.to_str().ok()?;
                    Some(csrf.to_string())
                }
                CsrfVerify::Query(name) => {
                    let query = req.query_string().to_string().replace('?', "");
                    let mut query_value = query.split('&');

                    let csrf = query_value
                        .find(|v| v.starts_with(format!("{name}=").as_str()))
                        .map(|v| v.split('=').nth(1))?;

                    csrf.map(|i| i.to_string())
                }
                CsrfVerify::Body(_name) => {
                    todo!("figure out a way to extract data from the request body")
                }
            }
        } else {
            None
        }
    }
}

impl<S> Service<ServiceRequest> for VerifyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let route = req.match_pattern().unwrap_or_default();
        let mut error = None;
        let mut csrf = None;

        // Validate there is an active session
        match req.extensions().get::<Authenticated>() {
            Some(authenticated) => {
                if self.should_verify_csrf() {
                    csrf = Some(authenticated.session.csrf.clone());
                }
            }
            None => {
                log::debug!("auth::middleware::verify|csrf_mismatch|route={}", route);

                error = Some(AppError::Unauthorized("not_authenticated".to_string()));
            }
        };

        // Validate the csrf token
        if error.is_none() && self.should_verify_csrf() && self.extract_csrf(&req) != csrf {
            log::debug!("auth::middleware::verify|csrf_mismatch|route={}", route);

            error = Some(AppError::Unauthorized("csrf_mismatch".to_string()));
        }

        if let Some(error) = error {
            return Box::pin(async move { Err(error.into()) });
        }

        log::debug!("auth::middleware::verify|is_authenticated route={}", route);
        let fut = self.service.call(req);
        Box::pin(fut)
    }
}
