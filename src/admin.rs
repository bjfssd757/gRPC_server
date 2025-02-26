// admin service

mod proto {
    tonic::include_proto!("control");
}
mod schema;
mod models;
mod calendar;

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    main: Address,
    hook: Address,
    admin: Address,
}

#[derive(Serialize, Deserialize, Debug)]
struct Address {
    address: String,
    port: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Admin {
    password: String,
    header_name: String,
}

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let data = std::fs::read_to_string("settings/settings.json")?;

    let settings: Settings = serde_json::from_str(&data)?;

    let token: MetadataValue<_> = settings.admin.password.parse().unwrap();

    match req.metadata().get(settings.admin.header_name) {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token!")),
    }
}

#[derive(Debug, Default)]
pub struct AdminService {
    state: calendar::State,
}

#[tonic::async_trait]
impl Admin for AdminService {
    async fn get_request_count(
        &self,
        _: tonic::Request<proto::GetCountRequest>,
    ) -> Result<tonic::Response<proto::CounterResponse>, tonic::Status> {
        let count = self.state.read().await;
        let response = proto::CounterResponse {
            count: *count,
        };
        Ok(tonic::Response::new(response))
    }
}