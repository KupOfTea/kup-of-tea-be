use actix_identity::Identity;
use actix_multipart::Multipart;
use actix_web::{
    options, post,
    web::{self},
    HttpResponse, Responder,
};
use futures::{StreamExt, TryStreamExt};
use sha3::{Digest, Keccak256};
use std::{fs::File, io::Write};

#[options("/api/image/upload")]
pub async fn upload_cors2() -> impl Responder {
    HttpResponse::Ok().body("Ok")
}

#[post("/api/image/upload")]
pub async fn add_image_api(mut payload: Multipart, identity: Option<Identity>) -> HttpResponse {
    if identity.is_none() {
        return HttpResponse::Forbidden().body("Access Denied");
    }

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

    HttpResponse::Ok().json(files_hash)
}
