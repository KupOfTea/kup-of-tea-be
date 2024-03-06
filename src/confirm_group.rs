use actix_identity::Identity;
use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::{Pool, Transaction};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfirmInfo {
    name: String,
    logo: String,
    group_photo: String,
    ticker: String,
    gender: String,
    types: String,
    agency_id: i64,
    acronyms: String,
    artist_members: Vec<ArtistMemberInfo>,
}

#[derive(Deserialize)]
pub struct ArtistMemberInfo {
    name: String,
    profile_image: String,
}

#[post("/api/group/confirm")]
pub async fn confirm_group_api(
    identity: Option<Identity>,
    info: web::Json<ConfirmInfo>,
    db: web::Data<Pool>,
) -> impl Responder {
    if identity.is_none() {
        return HttpResponse::Forbidden().body("Access Denied");
    }

    let mut client = db.get().await.expect("Failed to get DB client.");

    let transaction = client
        .transaction()
        .await
        .expect("Failed to start transaction.");

    let team_id = create_team(&transaction, &info)
        .await
        .expect("Failed to create team.");

    for artist_member in &info.artist_members {
        create_artist_member(&transaction, artist_member, team_id)
            .await
            .expect("Failed to create artist member.");
    }

    delete_temp_team(&transaction, &info.name)
        .await
        .expect("Failed to delete temp team.");

    transaction
        .commit()
        .await
        .expect("Failed to commit transaction.");

    HttpResponse::Ok().body("Group confirmed successfully")
}

async fn create_team(
    transaction: &Transaction<'_>,
    info: &ConfirmInfo,
) -> Result<i64, Box<dyn std::error::Error>> {
    let stmt = "INSERT INTO teams (name, logo, group_photo, ticker, gender, type, agency_id, acronyms) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id";
    let id: i64 = transaction
        .query_one(
            stmt,
            &[
                &info.name,
                &info.logo,
                &info.group_photo,
                &info.ticker,
                &info.gender,
                &info.types,
                &info.agency_id,
                &info.acronyms,
            ],
        )
        .await?
        .get(0);
    Ok(id)
}

async fn create_artist_member(
    transaction: &Transaction<'_>,
    artist_member: &ArtistMemberInfo,
    team_id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let stmt = "INSERT INTO artist_members (name, profile_image, team_id) VALUES ($1, $2, $3)";
    transaction
        .execute(
            stmt,
            &[&artist_member.name, &artist_member.profile_image, &team_id],
        )
        .await?;
    Ok(())
}

async fn delete_temp_team(
    transaction: &Transaction<'_>,
    team_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let stmt = "DELETE FROM temp_teams WHERE name = $1";
    transaction.execute(stmt, &[&team_name]).await?;
    Ok(())
}
