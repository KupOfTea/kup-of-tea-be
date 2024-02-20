use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Group {
    name: String,
    logo: String,
}

#[get("/api/group/get/{ticker}")]
pub async fn get_group_api(db: web::Data<Pool>, path: web::Path<String>) -> impl Responder {
    let client = db.get().await.unwrap();

    let ticker = path.trim();

    let stmt_groups = client
        .prepare("SELECT name, logo FROM teams WHERE ticker = $1")
        .await
        .unwrap();

    let groups_result = client.query_one(&stmt_groups, &[&ticker]).await;

    match groups_result {
        Ok(group_row) => {
            let group = Group {
                name: group_row.get("name"),
                logo: group_row.get("logo"),
            };

            HttpResponse::Ok().json(group)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
