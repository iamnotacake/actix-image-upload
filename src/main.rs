use actix_web::{ post, App, HttpServer, Responder, web };

struct Config {
    host: String,
    port: u16,
    tmp_dir: PathBuf,
    uploads_dir: PathBuf,
}

#[post("/upload")]
async fn upload() -> impl Responder {
    web::HttpResponse::NotImplemented()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let config = Config {
        host: "127.0.0.1".into(),
        port: 8080,
        tmp_dir: "/tmp".into(),
        uploads_dir: "/tmp/uploads".into(),
    };

    tokio::fs::create_dir_all(&config.tmp_dir).await?;
    tokio::fs::create_dir_all(&config.uploads_dir).await?;

    HttpServer::new(|| {
        App::new()
            .service(upload)
    })
    .bind((config.host.as_ref(), config.port))?
    .run()
    .await
}
