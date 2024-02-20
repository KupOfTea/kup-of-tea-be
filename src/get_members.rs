use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Member {
    name: String,
    profile_image: String,
}

#[get("/api/members/get/{ticker}")]
pub async fn get_members_api(db: web::Data<Pool>, path: web::Path<String>) -> impl Responder {
    let client = db.get().await.unwrap();

    let ticker = path.trim();

    let stmt_groups = client
        .prepare(
            "SELECT am.name, am.profile_image
            FROM artist_members am
            JOIN teams t ON am.team_id = t.id
            WHERE t.ticker = $1;
        ",
        )
        .await
        .unwrap();

    let members_result = client.query(&stmt_groups, &[&ticker]).await;

    match members_result {
        Ok(member_rows) => {
            let mut members = Vec::<Member>::new();
            for member_row in member_rows {
                let member = Member {
                    name: member_row.get("name"),
                    profile_image: member_row.get("profile_image"),
                };

                members.push(member);
            }

            HttpResponse::Ok().json(members)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
