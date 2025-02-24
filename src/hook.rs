use actix_web::{post, App, HttpResponse, HttpServer, Responder};

#[post("/ds")]
async fn ii(req: String) -> impl Responder {
    println!("Received request: {}", req);
    HttpResponse::Ok().body(format!{"Hey!"})
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on 0.0.0.0:5050");
    HttpServer::new(|| App::new()
        .service(ii))
        .bind(("10.6.0.20/24", 5050))?
        .run()
        .await
}