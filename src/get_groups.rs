use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Group {
    name: String,
    logo: String,
    ticker: String,
}

#[get("/api/groups/get/{type}/{gender}")]
pub async fn get_groups_api(
    db: web::Data<Pool>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let client = db.get().await.unwrap();

    let gender = path.1.trim();

    if gender != "boy" && gender != "girl" && gender != "all" {
        return HttpResponse::InternalServerError().finish();
    }

    let group_type = path.0.trim();

    if group_type != "group" && group_type != "solo" {
        return HttpResponse::InternalServerError().finish();
    }

    let groups_result = if gender == "all" {
        let stmt_groups = client
            .prepare("SELECT name, logo, ticker FROM teams WHERE type = $1")
            .await
            .unwrap();

        client.query(&stmt_groups, &[&group_type]).await
    } else {
        let stmt_groups = client
            .prepare("SELECT name, logo, ticker FROM teams WHERE gender = $1 AND type = $2")
            .await
            .unwrap();

        client.query(&stmt_groups, &[&gender, &group_type]).await
    };

    match groups_result {
        Ok(group_rows) => {
            let mut groups = Vec::<Group>::new();

            for group_row in group_rows {
                let group = Group {
                    name: group_row.get("name"),
                    logo: group_row.get("logo"),
                    ticker: group_row.get("ticker"),
                };

                groups.push(group);
            }

            groups.sort_by_key(|group| group.name.clone());

            HttpResponse::Ok().json(groups)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
