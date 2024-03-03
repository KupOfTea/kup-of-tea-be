use actix_multipart::Multipart;
use actix_web::{
    options, post,
    web::{self},
    HttpRequest, HttpResponse, Responder,
};
use chrono::{NaiveDateTime, Utc};
use deadpool_postgres::Pool;
use futures::{StreamExt, TryStreamExt};
use josekit::{
    jwe::ECDH_ES,
    jwt::{self},
};
use reqwest::Client;
use serde_json;
use sha3::{Digest, Keccak256};
use std::{env, fs::File, io::Write};

#[post("/api/upload/{path}")]
pub async fn upload_api(
    mut payload: Multipart,
    req: HttpRequest,
    db: web::Data<Pool>,
    path: web::Path<String>,
) -> HttpResponse {
    // 이미지 수 제한 변수
    let mut image_count = 0;

    // get image
    while let Ok(Some(mut field)) = payload.try_next().await {
        if image_count >= 20 {
            break;
        }
        image_count += 1;

        // file name and extension
        let content_type = field.content_disposition();
        let filename = content_type.get_filename().unwrap().to_string();
        let extension = filename
            .split('.')
            .last()
            .unwrap()
            .to_string()
            .to_lowercase();

        if !["jpg", "jpeg", "png", "gif", "webp"].contains(&extension.as_str()) {
            let json_data = serde_json::json!({
                "notimg": true
            });
            return HttpResponse::Accepted().json(json_data);
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

        // get image size
        let size = match imagesize::blob_size(&f) {
            Ok(size) => size,
            Err(_) => {
                return HttpResponse::Forbidden().body("incompleted upload");
            }
        };

        // get hash value
        let mut hasher = Keccak256::new();
        hasher.update(&f);
        let result = hasher.finalize();
        let hash_value = format!("{:x}", result);

        // get file path
        let filepath = format!("./static/image/{}.{}", hash_value, extension);

        // 파일 저장
        let mut file = File::create(filepath).unwrap();
        file.write_all(&f).unwrap();

        // 중복 검사
        let stmt = client
            .prepare("SELECT EXISTS (SELECT 1 FROM images WHERE img_hash = $1);")
            .await
            .unwrap();
        let rows = client.query_one(&stmt, &[&hash_value]).await.unwrap();
        let exists: bool = rows.get(0);

        // 중복일시 무시, 신규일시 등록
        if exists {
            // do nothing
        } else {
            let transaction = client.transaction().await.unwrap();

            let stmt = transaction
                .prepare("INSERT INTO images (img_hash, extension, ipv4, width, height) VALUES ($1, $2, $3, $4, $5);")
                .await
                .unwrap();
            let _result = transaction
                .execute(
                    &stmt,
                    &[
                        &hash_value,
                        &extension,
                        &ip_addr,
                        &(size.width as i32),
                        &(size.height as i32),
                    ],
                )
                .await
                .unwrap();

            let stmt = transaction
                .prepare(
                    "
                    INSERT INTO ip (ipv4, country, provider)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (ipv4)
                    DO UPDATE SET upload_count = ip.upload_count + 1;
                ",
                )
                .await
                .unwrap();

            let _result = transaction
                .execute(&stmt, &[&ip_addr, &guard.iso_code, &guard.provier])
                .await
                .unwrap();

            transaction.commit().await.unwrap();
        }
    }

    HttpResponse::Ok().body("Ok")
}
