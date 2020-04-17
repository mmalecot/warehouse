use crate::{
    core::error::{WarehouseError, WarehouseResult},
    database::PooledConnection,
    service::user::model::User,
    utils::regex::Regexes,
    view,
};
use actix_identity::Identity;
use actix_web::{
    http::header::LOCATION,
    web::{Data, Form},
    HttpRequest, HttpResponse,
};
use bcrypt::DEFAULT_COST;
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SignInForm {
    login: String,
    password: String,
}

#[derive(Deserialize)]
pub struct SignUpForm {
    username: String,
    email: String,
    password: String,
    password_confirmation: String,
}

impl SignUpForm {
    pub fn is_valid(&self, regexes: &Regexes) -> bool {
        regexes.username.is_match(&self.username)
            && regexes.email.is_match(&self.email)
            && regexes.password.is_match(&self.password)
            && self.password == self.password_confirmation
    }
}

pub async fn handle_sign_in_post(
    connection: PooledConnection,
    form: Form<SignInForm>,
    identity: Identity,
    request: HttpRequest,
) -> WarehouseResult<HttpResponse> {
    if let Some(user) = User::find_by_name_or_email(&connection, &form.login)? {
        if bcrypt::verify(&form.password, &user.password)? {
            identity.remember(user.id);
            Ok(HttpResponse::Found().header(LOCATION, "/").finish())
        } else {
            view!(&request, "route/user/sign_in", ["error" => "Incorrect password."])
        }
    } else {
        view!(&request, "route/user/sign_in", ["error" => "Incorrect username or email."])
    }
}

pub async fn handle_sign_out_post(identity: Identity) -> WarehouseResult<HttpResponse> {
    identity.forget();
    Ok(HttpResponse::Found().header(LOCATION, "/").finish())
}

pub async fn handle_sign_up_post(
    connection: PooledConnection,
    form: Form<SignUpForm>,
    identity: Identity,
    regexes: Data<Regexes>,
    request: HttpRequest,
) -> WarehouseResult<HttpResponse> {
    if form.is_valid(&regexes) {
        if User::exists(&connection, &form.username, &form.email)? {
            view!(&request, "route/user/sign_up", ["error" => "User already exists with that name or email."])
        } else {
            let user = User {
                id: Uuid::new_v4().to_string(),
                creation_date: Utc::now().naive_utc(),
                name: form.username.clone(),
                email: form.email.clone(),
                password: bcrypt::hash(&form.password, DEFAULT_COST)?,
                admin: User::count(&connection)? == 0,
            };
            user.create(&connection)?;
            identity.remember(user.id);
            Ok(HttpResponse::Found().header(LOCATION, "/").finish())
        }
    } else {
        Err(WarehouseError::InvalidFormData)
    }
}

pub async fn serve_sign_in_page(request: HttpRequest) -> WarehouseResult<HttpResponse> {
    view!(&request, "route/user/sign_in")
}

pub async fn serve_sign_up_page(request: HttpRequest) -> WarehouseResult<HttpResponse> {
    view!(&request, "route/user/sign_up")
}
