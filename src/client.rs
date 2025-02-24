use std::error::Error;
use proto::calendar_client::CalendarClient;

pub mod proto {
    tonic::include_proto!("hello");
    tonic::include_proto!("calendar");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://localhost:50051/".to_string();

    let mut client = CalendarClient::connect(url).await?;
    let req = proto::GetEventsRequest {};
    let request = tonic::Request::new(req);
    //request.metadata_mut().insert("authorization", "some-secret-token".parse()?);
    let response = client.get_events(request).await?;

    println!("RESPONSE:\n{:?}\n", response.get_ref());

    println!("\nEVENT 1:\nid = {:?}\nname = {:?}\nfulltime = {:?}\ndate = {:?}\ncreate at = {:?}\nlocation = {:?}\nauthor = {:?}\n", &response.get_ref().events[0].id, &response.get_ref().events[0].name,
            &response.get_ref().events[0].fulltime, &response.get_ref().events[0].date, &response.get_ref().events[0].create_at, &response.get_ref().events[0].location, &response.get_ref().events[0].author);

    Ok(())
}