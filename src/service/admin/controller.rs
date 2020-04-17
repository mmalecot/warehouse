use crate::{core::error::WarehouseResult, utils::auth::Authentication, view};
use actix_web::{HttpRequest, HttpResponse};

pub async fn serve_admin_page(
    auth: Authentication,
    request: HttpRequest,
) -> WarehouseResult<HttpResponse> {
    view!(&request, "route/admin", ["user" => &auth.user()])
}
