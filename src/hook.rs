use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    main: Address,
    hook: Address,
}

#[derive(Serialize, Deserialize, Debug)]
struct Address {
    address: String,
    port: String,
}

#[post("/ds")]
async fn ii(req: String) -> impl Responder {
    println!("Received request: {}", req);
    HttpResponse::Ok().body(format!{"Hey!"})
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = std::fs::read_to_string("settings/settings.json")?;

    let settings: Settings = serde_json::from_str(&data)?;

    HttpServer::new(|| App::new()
        .service(ii))
        .bind(format!("{}:{}", settings.hook.address, settings.hook.port))?
        .run()
        .await
}