#![allow(clippy::type_complexity)]

use crate::{core::error::WarehouseError, database::Pool, service::user::model::User};
use actix_identity::RequestIdentity;
use actix_service::{Service, Transform};
use actix_web::{
    dev::{Payload, ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    http::header::LOCATION,
    Error, FromRequest, HttpMessage, HttpRequest, HttpResponse, Result,
};
use futures::future::{self, Either, Ready};
use std::{
    convert::TryFrom,
    task::{Context, Poll},
};

#[derive(Clone)]
pub struct Authenticate {
    redirect_to: String,
}

impl Authenticate {
    pub fn new(redirect_to: &str) -> Authenticate {
        Authenticate {
            redirect_to: redirect_to.to_string(),
        }
    }
}

impl<S, B> Transform<S> for Authenticate
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticateMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(AuthenticateMiddleware {
            service,
            redirect_to: self.redirect_to.to_string(),
        })
    }
}

pub struct AuthenticateMiddleware<S> {
    service: S,
    redirect_to: String,
}

impl<S, B> Service for AuthenticateMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(context)
    }

    fn call(&mut self, request: ServiceRequest) -> Self::Future {
        if request.path() == self.redirect_to || Self::is_authenticated(&request) {
            Either::Left(self.service.call(request))
        } else {
            Either::Right(future::ok(
                request.into_response(
                    HttpResponse::Found()
                        .header(LOCATION, self.redirect_to.as_str())
                        .finish()
                        .into_body(),
                ),
            ))
        }
    }
}

impl<S> AuthenticateMiddleware<S> {
    fn is_authenticated(request: &ServiceRequest) -> bool {
        request.extensions().get::<AuthenticationItem>().is_some()
    }
}

pub struct Authentication(HttpRequest);

impl Authentication {
    pub fn user(&self) -> Option<User> {
        if let Some(auth) = self.0.extensions().get::<AuthenticationItem>() {
            Some(auth.user.clone())
        } else {
            None
        }
    }
}

impl FromRequest for Authentication {
    type Error = Error;
    type Future = Ready<Result<Authentication, Error>>;
    type Config = ();

    #[inline]
    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        future::ok(Authentication(request.clone()))
    }
}

struct AuthenticationItem {
    user: User,
}

pub struct AuthenticationService;

impl<S, B> Transform<S> for AuthenticationService
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationServiceMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(AuthenticationServiceMiddleware { service })
    }
}

pub struct AuthenticationServiceMiddleware<S> {
    service: S,
}

impl<S, B> Service for AuthenticationServiceMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(context)
    }

    fn call(&mut self, request: ServiceRequest) -> Self::Future {
        if let Err(error) = Self::authenticate(&request) {
            Either::Right(future::ok(request.error_response(error)))
        } else {
            Either::Left(self.service.call(request))
        }
    }
}

impl<S> AuthenticationServiceMiddleware<S> {
    fn authenticate(request: &ServiceRequest) -> Result<(), WarehouseError> {
        if let Some(id) = request.get_identity() {
            if let Some(pool) = request.app_data::<Pool>() {
                let connection = pool.get()?;
                if let Some(user) = User::find_by_id(&connection, &id)? {
                    request.extensions_mut().insert(AuthenticationItem { user });
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Authorize {
    redirect_to: String,
}

impl Authorize {
    pub fn new(redirect_to: &str) -> Authorize {
        Authorize {
            redirect_to: redirect_to.to_string(),
        }
    }
}

impl<S, B> Transform<S> for Authorize
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthorizeMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(AuthorizeMiddleware {
            service,
            redirect_to: self.redirect_to.clone(),
        })
    }
}

pub struct AuthorizeMiddleware<S> {
    service: S,
    redirect_to: String,
}

impl<S, B> Service for AuthorizeMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(context)
    }

    fn call(&mut self, request: ServiceRequest) -> Self::Future {
        if Self::is_authorized(&request) {
            Either::Left(self.service.call(request))
        } else {
            Either::Right(future::ok(
                request.into_response(
                    HttpResponse::Found()
                        .header(LOCATION, self.redirect_to.as_str())
                        .finish()
                        .into_body(),
                ),
            ))
        }
    }
}

impl<S> AuthorizeMiddleware<S> {
    fn is_authorized(request: &ServiceRequest) -> bool {
        request
            .extensions()
            .get::<AuthenticationItem>()
            .map_or(false, |auth| auth.user.admin)
    }
}

#[derive(Clone)]
pub struct RedirectIfAuthenticated {
    redirect_to: String,
}

impl RedirectIfAuthenticated {
    pub fn new(redirect_to: &str) -> RedirectIfAuthenticated {
        RedirectIfAuthenticated {
            redirect_to: redirect_to.to_string(),
        }
    }
}

impl<S, B> Transform<S> for RedirectIfAuthenticated
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RedirectIfAuthenticatedMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(RedirectIfAuthenticatedMiddleware {
            service,
            redirect_to: self.redirect_to.clone(),
        })
    }
}

pub struct RedirectIfAuthenticatedMiddleware<S> {
    service: S,
    redirect_to: String,
}

impl<S, B> Service for RedirectIfAuthenticatedMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, context: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(context)
    }

    fn call(&mut self, request: ServiceRequest) -> Self::Future {
        if Self::is_authenticated(&request) {
            Either::Right(future::ok(
                request.into_response(
                    HttpResponse::Found()
                        .header(LOCATION, self.redirect_to.as_str())
                        .finish()
                        .into_body(),
                ),
            ))
        } else {
            Either::Left(self.service.call(request))
        }
    }
}

impl<S> RedirectIfAuthenticatedMiddleware<S> {
    fn is_authenticated(request: &ServiceRequest) -> bool {
        request
            .extensions()
            .get::<AuthenticationItem>()
            .map_or(false, |auth| auth.user.admin)
    }
}

impl FromRequest for User {
    type Error = Error;
    type Future = Ready<Result<User, Error>>;
    type Config = ();

    fn from_request(request: &HttpRequest, _: &mut Payload) -> Self::Future {
        match User::try_from(request) {
            Ok(user) => future::ok(user),
            Err(error) => future::err(error),
        }
    }
}

impl TryFrom<&HttpRequest> for User {
    type Error = Error;

    fn try_from(request: &HttpRequest) -> Result<Self, Self::Error> {
        if let Some(auth) = request.extensions().get::<AuthenticationItem>() {
            Ok(auth.user.clone())
        } else {
            Err(ErrorUnauthorized("Unauthorized"))
        }
    }
}
