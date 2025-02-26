use dotenv::dotenv;
use tonic::transport::Server;
use tonic::Status;
use serde::{Deserialize, Serialize};
use proto::{calendar_server, control_server};

mod proto {
    tonic::include_proto!("calendar");
    tonic::include_proto!("control");
}
mod calendar;
mod admin;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn  std::error::Error>> {
    dotenv().ok();

    let data = std::fs::read_to_string("settings/settings.json")?;

    let settings: Settings = serde_json::from_str(&data)?;

    let addr = format!("{}:{}", settings.main.address, settings.main.port)
        .parse()?;

    let state = calendar::State::default();

    let admin = admin::AdminService{
        state: state.clone(),
    };
    let calndr = calendar::CalendarService {
        state: state.clone(),
    };

    Server::builder()
        //.add_service(AdminServer::with_interceptor(admin, check_auth))
        //.add_service(HelloServer::new(hello))
        .add_service(calendar_server::CalendarServer::new(calndr))
        .serve(addr)
        .await?;
    
    Ok(())
}