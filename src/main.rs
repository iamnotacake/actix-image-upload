use std::fmt;

use actix_multipart::Multipart;
use actix_web::{guard, web, App, FromRequest, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use tokio::stream::StreamExt;

use lib::{Config, UploadedFile};
use actix_image_upload as lib;

async fn upload_multipart(mut multipart: Multipart, config: web::Data<Config>) -> impl Responder {
    let mut uploaded_files = Vec::new();

    while let Ok(Some(field)) = multipart.try_next().await {
        let extension = match lib::mime_type_to_extension(field.content_type().essence_str()) {
            Some(extension) => extension,
            None => return web::HttpResponse::UnsupportedMediaType(),
        };

        let res = lib::upload_image(field, &config.get_ref().uploads_dir, extension).await;
        match res {
            Ok(uploaded_file) => {
                log::info!(
                    "Upload succeed, id: {}, path: {}, thumbnail: {}",
                    uploaded_file.id,
                    uploaded_file.path.to_str().unwrap_or("?"),
                    if let Some(ref path) = uploaded_file.thumbnail_path {
                        path.to_str().unwrap_or("?")
                    } else {
                        "Failed to create"
                    },
                );

                uploaded_files.push(uploaded_file);
            }
            Err(err) => {
                log::error!("Upload error: {}", err);

                if let Some(lib::UploadError::Client(_)) = err.downcast_ref() {
                    return web::HttpResponse::BadRequest();
                } else {
                    return web::HttpResponse::InternalServerError();
                }
            }
        }
    }

    if !uploaded_files.is_empty() {
        log::info!(
            "Uploaded {} file{} in total (multipart/form-data)",
            uploaded_files.len(),
            if uploaded_files.len() > 1 { "s" } else { "" },
        );

        return web::HttpResponse::Ok();
    } else {
        return web::HttpResponse::BadRequest();
    }
}

#[derive(Deserialize)]
enum UploadRequest {
    #[serde(rename = "url")]
    Url(String),
    #[serde(rename = "base64")]
    Base64(String),
}

impl fmt::Debug for UploadRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UploadRequest::Url(url) => write!(f, "Url(\"{}\")", url),
            UploadRequest::Base64(data) => write!(f, "Base64({} bytes)", data.len()),
        }
    }
}

async fn upload_json(
    req: web::Json<Vec<UploadRequest>>,
    config: web::Data<Config>,
) -> impl Responder {
    let mut uploaded_files: Vec<UploadedFile> = Vec::new();

    for item in req.iter() {
        log::debug!("{:?}", item)
    }

    for upload_request in req.iter() {
        match upload_request {
            UploadRequest::Url(url) => {
                let res = lib::fetch_image(&config.get_ref(), &url).await;
                match res {
                    Ok(uploaded_file) => {
                        log::info!(
                            "Upload succeed, id: {}, path: {}, thumbnail: {}",
                            uploaded_file.id,
                            uploaded_file.path.to_str().unwrap_or("?"),
                            if let Some(ref path) = uploaded_file.thumbnail_path {
                                path.to_str().unwrap_or("?")
                            } else {
                                "Failed to create"
                            },
                        );

                        uploaded_files.push(uploaded_file);
                    }
                    Err(err) => {
                        log::error!("Upload error: {}", err);

                        if let Some(lib::UploadError::Client(_)) = err.downcast_ref() {
                            return web::HttpResponse::BadRequest();
                        } else {
                            return web::HttpResponse::InternalServerError();
                        }
                    }
                }
            }
            UploadRequest::Base64(data) => match base64::decode(&data) {
                Ok(data) => {
                    let content_type = tree_magic::from_u8(&data);
                    log::debug!("{}", &content_type);

                    let extension = match lib::mime_type_to_extension(&content_type) {
                        Some(extension) => extension,
                        None => return web::HttpResponse::UnsupportedMediaType(),
                    };

                    let data = bytes::Bytes::from(data);
                    let stream = tokio::stream::once(Ok::<_, failure::Error>(data));
                    let res =
                        lib::upload_image(stream, &config.get_ref().uploads_dir, extension).await;
                    match res {
                        Ok(uploaded_file) => {
                            log::info!(
                                "Upload succeed, id: {}, path: {}, thumbnail: {}",
                                uploaded_file.id,
                                uploaded_file.path.to_str().unwrap_or("?"),
                                if let Some(ref path) = uploaded_file.thumbnail_path {
                                    path.to_str().unwrap_or("?")
                                } else {
                                    "Failed to create"
                                },
                            );

                            uploaded_files.push(uploaded_file);
                        }
                        Err(err) => {
                            log::error!("Upload error: {}", err);

                            if let Some(lib::UploadError::Client(_)) = err.downcast_ref() {
                                return web::HttpResponse::BadRequest();
                            } else {
                                return web::HttpResponse::InternalServerError();
                            }
                        }
                    }
                }
                Err(err) => {
                    log::error!("Base64 decode error: {}", err);

                    return web::HttpResponse::BadRequest();
                }
            },
        }
    }

    if !uploaded_files.is_empty() {
        log::info!(
            "Uploaded {} file{} in total (application/json)",
            uploaded_files.len(),
            if uploaded_files.len() > 1 { "s" } else { "" },
        );

        return web::HttpResponse::Ok();
    } else {
        return web::HttpResponse::BadRequest();
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = Config {
        host: "0.0.0.0".into(),
        port: 8080,
        uploads_dir: "/tmp/uploads".into(),
        max_json_payload_size: 1 << 20,
    };

    tokio::fs::create_dir_all(&config.uploads_dir).await?;

    let (host, port) = (config.host.clone(), config.port);

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .app_data(web::Json::<Vec<UploadRequest>>::configure(|cfg| {
                cfg.limit(config.max_json_payload_size)
            }))
            .service(
                web::scope("/upload")
                    .guard(guard::Post())
                    .guard(guard::fn_guard(|req| {
                        if let Some(content_type) = req.headers().get("content-type") {
                            if let Ok(s) = content_type.to_str() {
                                s.starts_with("multipart/form-data;")
                            } else { false }
                        } else { false }
                    }))
                    .route("", web::post().to(upload_multipart)),
            )
            .service(
                web::scope("/upload")
                    .guard(guard::Post())
                    .guard(guard::fn_guard(|req| {
                        if let Some(content_type) = req.headers().get("content-type") {
                            if let Ok(s) = content_type.to_str() {
                                s == "application/json"
                            } else { false }
                        } else { false }
                    }))
                    .route("", web::post().to(upload_json)),
            )
            // Handle application/x-www-form-urlencoded ?
            .service(
                web::scope("/upload")
                    .route("", web::to(|| HttpResponse::BadRequest()))
            )
    })
    .bind((host.as_ref(), port))?
    .run()
    .await
}
