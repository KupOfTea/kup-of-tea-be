use actix_identity::Identity;
use actix_web::{
    post,
    web::{self},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use deadpool_postgres::Pool;
#[derive(serde::Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

#[post("/api/login")]
pub async fn login_api(
    info: web::Form<LoginInfo>,
    req: HttpRequest,
    db: web::Data<Pool>,
) -> impl Responder {
    let client = db.get().await.unwrap();
    let stmt = client
        .prepare("SELECT pw FROM users WHERE id = $1")
        .await
        .unwrap();
    let row = client.query_one(&stmt, &[&info.username]).await.unwrap();

    let pw_hash: String = row.get("pw");
    let parsed_hash = PasswordHash::new(&pw_hash).unwrap();

    let result = Argon2::default().verify_password(&info.password.as_bytes(), &parsed_hash);

    match result {
        Ok(_result) => {
            Identity::login(&req.extensions(), info.username.to_owned()).unwrap();
        }
        Err(_) => return HttpResponse::Forbidden().body("No"),
    }

    HttpResponse::Ok().body("Ok")
}
