use actix_multipart::Multipart;
use actix_web::{
    options, post,
    web::{self},
    HttpResponse, Responder,
};
use deadpool_postgres::Pool;
use futures::{StreamExt, TryStreamExt};
use sha3::{Digest, Keccak256};
use std::{fs::File, io::Write};

#[options("/api/group/upload/{name}")]
pub async fn upload_cors(_path: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body("Ok")
}

#[post("/api/group/upload/{name}")]
pub async fn add_group_api(
    mut payload: Multipart,
    db: web::Data<Pool>,
    path: web::Path<String>,
) -> HttpResponse {
    let mut image_count = 0;

    let mut files_hash: Vec<String> = vec![];

    while let Ok(Some(mut field)) = payload.try_next().await {
        if image_count >= 20 {
            break;
        }
        image_count += 1;

        let content_type = field.content_disposition();
        let filename = content_type.get_filename().unwrap().to_string();
        let extension = filename
            .split('.')
            .last()
            .unwrap()
            .to_string()
            .to_lowercase();

        if !["jpg", "jpeg", "png", "gif", "webp"].contains(&extension.as_str()) {
            return HttpResponse::Forbidden().body("non-image upload");
        }

        let mut f = web::BytesMut::new();
        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(chunk) => chunk,
                Err(_) => {
                    return HttpResponse::Forbidden().body("incompleted upload");
                }
            };
            f.extend_from_slice(&data);
        }

        let mut hasher = Keccak256::new();
        hasher.update(&f);
        let result = hasher.finalize();
        let hash_value = format!("{:x}", result);

        let filepath = format!("./static/image/{}.{}", hash_value, extension);

        let mut file = File::create(filepath).unwrap();
        file.write_all(&f).unwrap();

        files_hash.push(format!("{}.{}", hash_value, extension));
    }

    let client = db.get().await.unwrap();

    let name = path.trim().to_string();

    let stmt = client
        .prepare("INSERT INTO temp_teams (name, team_photos) VALUES ($1, $2)")
        .await
        .unwrap();

    let team_photos: Vec<&str> = files_hash.iter().map(|x| x.as_str()).collect();

    let _row = client.execute(&stmt, &[&name, &team_photos]).await.unwrap();

    HttpResponse::Ok().body("Ok")
}
