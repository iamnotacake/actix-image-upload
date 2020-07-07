use std::path::PathBuf;
use actix_web::{ post, App, HttpServer, Responder, web };
use actix_multipart::Multipart;
use tokio::prelude::*;
use tokio::stream::StreamExt;
use actix_image_upload as lib;

#[derive(Clone)]
struct Config {
    host: String,
    port: u16,
    uploads_dir: PathBuf,
}

#[post("/upload")]
async fn upload(mut multipart: Multipart, config: web::Data<Config>) -> impl Responder {
    if let Ok(Some(field)) = multipart.try_next().await {
        if field.content_type().type_() != mime::IMAGE {
            return web::HttpResponse::UnsupportedMediaType()
        }

        let extension = match field.content_type().subtype().as_str() {
            "bmp" => "bmp",
            "jpeg" => "jpg",
            "png" => "png",
            _ => return web::HttpResponse::UnsupportedMediaType(),
        };

        let content_disposition = field.content_disposition().unwrap();

        if content_disposition.get_name() != Some("image") {
            return web::HttpResponse::BadRequest();
        }

        let res = lib::upload_image(field, &config.get_ref().uploads_dir, extension).await;
        match res {
            Ok(lib::UploadedFile { id, path, thumbnail_path }) => {
                eprintln!(
                    "Upload succeed, id: {}, path: {}, thumbnail: {}",
                    id,
                    path.to_str().unwrap_or("?"),
                    if let Some(ref path) = thumbnail_path { path.to_str().unwrap_or("?") } else { "Failed to create" },
                );
                return web::HttpResponse::Ok();
            }
            Err(lib::UploadError::Client(e)) => {
                eprintln!("Upload error: {}", e);
                return web::HttpResponse::BadRequest();
            }
            Err(lib::UploadError::Server(e)) => {
                eprintln!("Upload error: {}", e);
                return web::HttpResponse::InternalServerError();
            }
        }
    }

    web::HttpResponse::BadRequest()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let config = Config {
        host: "127.0.0.1".into(),
        port: 8080,
        uploads_dir: "/tmp/uploads".into(),
    };

    tokio::fs::create_dir_all(&config.uploads_dir).await?;

    let (host, port) = (config.host.clone(), config.port);

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .service(upload)
    })
    .bind((host.as_ref(), port))?
    .run()
    .await
}
