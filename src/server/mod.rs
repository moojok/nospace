//! # Application server
//!
//! From this module we define all the application HTTP routes and start the server.
//! This module and its sub-modules will give you a good idea of the application endpoints
//! and endpoint Request and Response structs.

use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web, App, HttpServer,
};
use context::Context;

pub mod middleware {
    //! # Middleware
    //!
    //! Collection of all the middleware used in the application pulled
    //! from various packages we depend on.
    pub use ::auth::middleware::{load::Load, verify::Verify};
}

pub mod api;
pub mod cors;
pub mod proxy;

/// Create the web application and inject all the routes into it
pub fn app(
    context: Context,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<EitherBody<BoxBody>>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let mut auth_load_middleware = middleware::Load::new()
        .add_ignore("/api/auth/register".to_string())
        .add_ignore("/api/auth/login".to_string())
        .add_ignore("/api/auth/signature".to_string());

    if context.config.use_cookies {
        let cookie_name = context.config.get_cookie_name();
        auth_load_middleware = auth_load_middleware.token_cookie_name(cookie_name);
    }

    App::new()
        .app_data(web::Data::new(context))
        // Authentication load middleware that only sets it up on the app
        .wrap(auth_load_middleware)
        .wrap(cors::setup())
        // PRETTY PLEASE: keep the routes in alphabetical order
        //  There is a VSCode extension "Alphabetical Sorter" that can help you with this
        .service(api::auth::authenticated_self)
        .service(api::auth::login)
        .service(api::auth::refresh)
        .service(api::auth::register)
        .service(api::auth::signature)
        // Proxy HTTP requests to frontend
        .route("/{tail:.*}", web::to(proxy::http))
}

/// Start the server
pub async fn engage(context: Context) -> std::io::Result<()> {
    let bind_address = context.config.get_full_bind_address();

    HttpServer::new(move || app(context.clone()))
        .bind(&bind_address)?
        .run()
        .await
}
