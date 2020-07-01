use actix_web::{ post, App, HttpServer, Responder, web };

#[post("/upload")]
async fn upload() -> impl Responder {
    web::HttpResponse::NotImplemented()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(upload)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
