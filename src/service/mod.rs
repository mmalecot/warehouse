use crate::utils::{
    auth::{Authenticate, Authorize, RedirectIfAuthenticated},
    path,
};
use actix_files::Files;
use actix_web::web::{self, ServiceConfig};

pub mod admin;
pub mod index;
pub mod package;
pub mod repository;
pub mod user;

pub fn configure(config: &mut ServiceConfig) {
    let authenticate = Authenticate::new("/user/sign_in");
    let authorize = Authorize::new("/");
    let redirect_if_authenticated = RedirectIfAuthenticated::new("/");
    config
        .service(Files::new("/static", path::static_files_dir()))
        .service(web::resource("/").route(web::get().to(index::controller::serve_index_page)))
        .service(
            web::resource("/favicon.ico").route(web::get().to(index::controller::serve_favicon)),
        )
        .service(
            web::resource("/admin")
                .wrap(authorize.clone())
                .wrap(authenticate.clone())
                .route(web::get().to(admin::controller::serve_admin_page)),
        )
        .service(
            web::scope("/package")
                .service(
                    web::resource("/{repository}/{architecture}/{name}.{extension:pkg.*}")
                        .route(web::get().to(package::controller::serve_package_archive)),
                )
                .service(
                    web::resource("/{repository}/{architecture}/{name}.{extension:(db|files)}")
                        .route(web::get().to(package::controller::serve_repository_database)),
                )
                .service(
                    web::resource("/{repository}/{architecture}/{name}")
                        .route(web::get().to(package::controller::fetch_package)),
                )
                .service(
                    web::resource("/{repository}/{architecture}/{name}/delete")
                        .wrap(authorize)
                        .route(web::post().to(package::controller::delete_package)),
                )
                .service(
                    web::resource("/import")
                        .wrap(authenticate)
                        .route(web::get().to(package::controller::serve_import_package_page))
                        .route(web::post().to(package::controller::handle_import_package_post)),
                )
                .service(
                    web::resource("/list")
                        .route(web::get().to(package::controller::serve_package_list_page)),
                ),
        )
        .service(
            web::scope("/user")
                .service(
                    web::resource("/sign_in")
                        .wrap(redirect_if_authenticated.clone())
                        .route(web::get().to(user::controller::serve_sign_in_page))
                        .route(web::post().to(user::controller::handle_sign_in_post)),
                )
                .service(
                    web::resource("/sign_out")
                        .route(web::post().to(user::controller::handle_sign_out_post)),
                )
                .service(
                    web::resource("/sign_up")
                        .wrap(redirect_if_authenticated)
                        .route(web::get().to(user::controller::serve_sign_up_page))
                        .route(web::post().to(user::controller::handle_sign_up_post)),
                ),
        );
}
