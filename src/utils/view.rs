use crate::{core::config::Config, utils::regex::Regexes};
use actix_web::HttpRequest;
use serde::Serialize;
use tera::Context;

#[derive(Serialize)]
struct AppContext {
    name: String,
    version: String,
}

#[derive(Serialize)]
struct RegexContext {
    email: String,
    password: String,
    username: String,
}

#[derive(Serialize)]
struct RequestContext {
    path: String,
}

pub struct ContextBuilder {
    context: Context,
}

impl ContextBuilder {
    pub fn build(self) -> Context {
        self.context
    }

    pub fn new() -> ContextBuilder {
        ContextBuilder {
            context: Context::new(),
        }
    }

    pub fn with_app(mut self) -> ContextBuilder {
        self.context.insert(
            "app",
            &AppContext {
                name: String::from(env!("CARGO_PKG_NAME")),
                version: String::from(env!("CARGO_PKG_VERSION")),
            },
        );
        self
    }

    pub fn with_config(mut self, config: &Config) -> ContextBuilder {
        self.context.insert("config", config);
        self
    }

    pub fn with_regexes(mut self, regexes: &Regexes) -> ContextBuilder {
        self.context.insert(
            "regexes",
            &RegexContext {
                email: regexes.email.to_string(),
                password: regexes.password.to_string(),
                username: regexes.username.to_string(),
            },
        );
        self
    }

    pub fn with_request(mut self, request: &HttpRequest) -> ContextBuilder {
        self.context.insert(
            "request",
            &RequestContext {
                path: String::from(request.path()),
            },
        );
        self
    }

    pub fn with_value<T: Serialize + ?Sized, S: Into<String>>(
        mut self,
        key: S,
        value: &T,
    ) -> ContextBuilder {
        self.context.insert(key, value);
        self
    }
}

#[macro_export]
macro_rules! view {
    ($request:expr, $view:expr) => ({
        view!($request, $view, [])
    });
    ($request:expr, $view:expr, [$($key:expr => $value:expr),*]) => ({
        use {
            crate::{
                core::config::Config,
                error::WarehouseError,
                utils::{regex::Regexes, view::ContextBuilder},
            },
            actix_web::web::Data,
            tera::Tera,
        };
        let config = $request
            .app_data::<Data<Config>>()
            .ok_or(WarehouseError::AppDataNotFound(String::from("Config")))?;
        let regexes = $request
            .app_data::<Data<Regexes>>()
            .ok_or(WarehouseError::AppDataNotFound(String::from("Regexes")))?;
        let template = $request
            .app_data::<Data<Tera>>()
            .ok_or(WarehouseError::AppDataNotFound(String::from("Tera")))?;
        let context = ContextBuilder::new()
            .with_app()
            .with_config(&config)
            .with_regexes(&regexes)
            .with_request($request)
            $(
                .with_value($key, $value)
            )*
            .build();
        Ok(HttpResponse::Ok().content_type("text/html").body(
            template
                .render(&format!("views/{}.html.tera", $view), &context)
                .map_err(WarehouseError::TeraError)?,
        ))
    });
}
