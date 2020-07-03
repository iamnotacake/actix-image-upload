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
    if let Ok(Some(mut field)) = multipart.try_next().await {
        if field.content_type().type_() != mime::IMAGE {
            return web::HttpResponse::UnsupportedMediaType()
        }

        let content_disposition = field.content_disposition().unwrap();

        if content_disposition.get_name() != Some("image") {
            return web::HttpResponse::BadRequest()
        }

        let mut tmp_path = PathBuf::with_capacity(64);
        tmp_path.push(&config.get_ref().uploads_dir);
        tmp_path.push(lib::gen_rand_id(12));
        tmp_path.set_extension("tmp");

        let file = tokio::fs::File::create(&tmp_path).await.unwrap();
        let mut writer = tokio::io::BufWriter::new(file);

        eprintln!("Uploading {} -> {}",
                  content_disposition.get_filename().unwrap_or("?"),
                  tmp_path.to_str().unwrap_or("?"));

        while let Some(chunk) = field.next().await {
            let chunk = chunk.unwrap();
            writer.write_all(&chunk).await.unwrap();
        }

        writer.flush().await.unwrap();

        let mut upload_path = tmp_path.clone();
        upload_path.set_extension("");

        eprintln!("Renaming {} -> {}", tmp_path.to_str().unwrap_or("?"), upload_path.to_str().unwrap_or("?"));
        tokio::fs::rename(&tmp_path, &upload_path).await.unwrap();

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
