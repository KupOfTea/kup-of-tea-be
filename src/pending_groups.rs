use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TempTeam {
    id: i64,
    name: String,
    team_photos: Vec<String>,
}

#[get("/api/group/pending/get")]
async fn get_pending_teams_api(db: web::Data<Pool>) -> impl Responder {
    let client = db.get().await.unwrap();

    let stmt = client
        .prepare("SELECT id, name, team_photos FROM temp_teams")
        .await
        .unwrap();

    let team_results = client.query(&stmt, &[]).await.unwrap();

    let temp_teams: Vec<TempTeam> = team_results
        .iter()
        .map(|row| TempTeam {
            id: row.get("id"),
            name: row.get("name"),
            team_photos: row.get("team_photos"),
        })
        .collect();

    HttpResponse::Ok().json(temp_teams)
}
