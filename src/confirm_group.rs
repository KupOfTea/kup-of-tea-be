use actix_identity::Identity;
use actix_web::{
    post,
    web::{self},
    HttpResponse, Responder,
};
use deadpool_postgres::Pool;

#[derive(serde::Deserialize)]
pub struct ConfirmInfo {
    id: i64,
    name: String,
    logo: String,
    group: String,
    ticker: String,
    gender: String,
    types: String,
    acronyms: String,
    team_photos: Vec<String>,
    agency_id: i64,
}

#[post("/api/delete")]
pub async fn confirm_group_api(
    identity: Option<Identity>,
    info: web::Form<ConfirmInfo>,
    db: web::Data<Pool>,
) -> impl Responder {
    let _id = match identity.map(|id| id.id()) {
        None => return HttpResponse::Forbidden().body("No"),
        Some(Ok(id)) => id,
        Some(Err(_)) => return HttpResponse::Forbidden().body("No"),
    };

    let client = db.get().await.unwrap();

    HttpResponse::Ok().body("Ok")
}
