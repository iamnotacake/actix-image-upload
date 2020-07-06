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

        let id = lib::gen_rand_id(12);

        let mut tmp_path = PathBuf::with_capacity(64);
        tmp_path.push(&config.get_ref().uploads_dir);
        tmp_path.push(&id);
        tmp_path.set_extension("tmp");

        let file = tokio::fs::File::create(&tmp_path).await.unwrap();
        let writer = tokio::io::BufWriter::new(file);

        eprintln!(
            "Uploading {} -> {}",
            content_disposition.get_filename().unwrap_or("?"),
            tmp_path.to_str().unwrap_or("?"),
        );

        let res = lib::stream_to_writer(field, writer).await;
        if let Err(err) = res {
            eprintln!("Upload error: {}", err);
            tokio::fs::remove_file(&tmp_path).await.unwrap();
            return web::HttpResponse::BadRequest();
        }

        let mut upload_path = tmp_path.clone();
        upload_path.set_extension(extension);

        eprintln!(
            "Renaming {} -> {}",
            tmp_path.to_str().unwrap_or("?"),
            upload_path.to_str().unwrap_or("?")
        );
        tokio::fs::rename(&tmp_path, &upload_path).await.unwrap();

        let mut thumbnail_path = upload_path.clone();
        thumbnail_path.set_file_name(format!("{}_thumbnail.{}", id, extension));

        eprintln!(
            "Thumbnail {} -> {}",
            upload_path.to_str().unwrap_or("?"),
            thumbnail_path.to_str().unwrap_or("?")
        );
        // Processing of a big image may be a hard task,
        // let's do it on a dedicated thread
        let res = tokio::task::spawn_blocking(move || {
            lib::imagetools::create_thumbnail(&upload_path, &thumbnail_path, (100, 100))
        }).await;

        if let Err(err) = res {
            eprintln!("{}", err);
            return web::HttpResponse::UnsupportedMediaType();
        }

        return web::HttpResponse::Ok();
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
