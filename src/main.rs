#[macro_use]
extern crate lazy_static;
extern crate diesel;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use schema::events::id;
use std::env;
//use proto::hello_server::{Hello, HelloServer};
//use proto::admin_server::{Admin, AdminServer};
use proto::calendar_server::{Calendar, CalendarServer};
//use tonic::metadata::MetadataValue;
use tonic::transport::Server;
//use tonic::{Request, Status};
use tonic::Status;
use serde::{Deserialize, Serialize};
//use self::models::{User, Event};
use self::models::Event;
//use crate::schema::users::dsl as users;
use crate::schema::events::dsl as events;
use prost_types::Timestamp;

mod proto {
    tonic::include_proto!("hello");
    tonic::include_proto!("control");
    tonic::include_proto!("calendar");
}
mod schema;
mod models;
mod hook;

lazy_static! {
    static ref POOL: r2d2::Pool<ConnectionManager<PgConnection>> = {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    };
}

fn get_connection() -> r2d2::PooledConnection<ConnectionManager<PgConnection>> {
    POOL.get().expect("Failed to get a connection from the pool")
}

/*fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "1234fb".parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token!")),
    }
}*/

fn system_time_to_timestamp(system_time: Option<std::time::SystemTime>) -> Timestamp {
    match system_time {
        Some(time) => {
            let duration = time.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            Timestamp {
                seconds: duration.as_secs() as i64,
                nanos: duration.subsec_nanos() as i32,
            }
        }
        None => Timestamp {
            seconds: 0,
            nanos: 0,
        }
    }
}

fn timestamp_to_system_time(timestamp: Option<Timestamp>) -> std::time::SystemTime {
    match timestamp {
        Some(time) => {
            std::time::UNIX_EPOCH + std::time::Duration::new(time.seconds as u64, time.nanos as u32)
        }
        None => std::time::SystemTime::UNIX_EPOCH
    }
}

//type State = std::sync::Arc<tokio::sync::RwLock<u64>>;

// ------------------------------------------------------------//
// Calendar Service

#[derive(Serialize, Deserialize, Debug)]
struct EventProto {
    id: i64,
    name: String,
    date: Option<std::time::SystemTime>,
    fulltime: bool,
    author: String,
    create_at: Option<std::time::SystemTime>,
    location: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventsList {
    events: Vec<EventProto>,
}

#[derive(Debug, Default)]
struct CalendarService;

impl CalendarService {
    async fn load_events() -> Result<EventsList, Box<dyn std::error::Error>> {
        let mut conn = get_connection();

        let all_events = events::events
            .load::<Event>(&mut conn)?;

        Ok(EventsList {events: all_events.into_iter().map(|event| EventProto {
            id: event.id.into(),
            name: event.name,
            date: Some(event.date),
            fulltime: event.fulltime,
            author: event.author,
            create_at: Some(event.create_at),
            location: event.location,
            message: event.message,
            }).collect()
        })
    }
}

#[tonic::async_trait]
impl Calendar for CalendarService {
    async fn get_events(
        &self,
        _: tonic::Request<proto::GetEventsRequest>,
        ) -> Result<tonic::Response<proto::GetEventsResponse>, Status> {
        let events_list = CalendarService::load_events().await.map_err(|e| {
            Status::internal(format!("Failed to load events: {}", e))
        })?;

        let events_p: Vec<proto::EventProto> = events_list.events.into_iter().map(|event| proto::EventProto {
            id: event.id,
            name: event.name,
            date: Some(system_time_to_timestamp(event.date)),
            fulltime: event.fulltime,
            author: event.author,
            create_at: Some(system_time_to_timestamp(event.create_at)),
            location: event.location,
            message: event.message,
        }).collect();

        let response = proto::GetEventsResponse { events: events_p };
        Ok(tonic::Response::new(response))
    }

    async fn get_event(
        &self,
        request: tonic::Request<proto::EventRequest>
        ) -> Result<tonic::Response<proto::EventResponse>, Status> {
        let events_list = CalendarService::load_events().await.map_err(|e| {
            Status::internal(format!("Failed to load events: {}", e))
        })?;

        let event_id = request.get_ref().id;

        let search_event = events_list.events.into_iter().find(|event| event.id == event_id);

        match search_event {
            Some(event) => {
                let proto_event = proto::EventProto {
                    id: event.id,
                    name: event.name,
                    date: Some(system_time_to_timestamp(event.date)),
                    fulltime: event.fulltime,
                    author: event.author,
                    create_at: Some(system_time_to_timestamp(event.create_at)),
                    location: event.location,
                    message: event.message,
                };
                let response = proto::EventResponse { event: Some(proto_event) };
                Ok(tonic::Response::new(response))
            },
            None => Err(Status::not_found("Event not found")),
            }
        }
    async fn add_event(
        &self,
        requset: tonic::Request<proto::AddEventRequest>
        ) -> Result<tonic::Response<proto::AddEventResponse>, Status> {
            let new_event = requset.get_ref().event.clone().ok_or_else(|| {
                Status::invalid_argument("Event is missing")
            })?;

            let events_list = CalendarService::load_events().await.map_err(|e| {
                Status::internal(format!("Failed to load events: {}", e))
            })?;

            if events_list.events.iter().any(|event| event.id == new_event.id) {
                return Err(Status::already_exists("Event with this ID already exists"));
            }
        
            let mut conn = get_connection();

            diesel::insert_into(events::events)
                .values(&Event {
                        id: new_event.id as i32,
                        name: new_event.name,
                        date: timestamp_to_system_time(new_event.date),
                        fulltime: new_event.fulltime,
                        author: new_event.author,
                        create_at: timestamp_to_system_time(new_event.create_at),
                        location: new_event.location,
                        message: new_event.message,                        
                    }
                )
                .execute(&mut conn)
                .map_err(|e| {
                    Status::internal(format!("Failed to save event: {}", e))
                })?;
                

            let response = proto::AddEventResponse { success: true, message: "201 Created".to_string()};
            Ok(tonic::Response::new(response))
        }
    
    async fn remove_event(
            &self,
            request: tonic::Request<proto::RemoveEventRequest>
        ) -> Result<tonic::Response<proto::RemoveEventResponse>, Status> {
            let event_id = request.get_ref().id;
        
            let events_list = CalendarService::load_events().await.map_err(|e| {
                Status::internal(format!("Failed to load events: {}", e))
            })?;
        
            let event_index = events_list.events.iter().position(|event| event.id == event_id);
        
            match event_index {
                Some(_) => {
                    let mut conn = get_connection();

                    diesel::delete(events::events.filter(id.eq(event_id as i32)))
                        .execute(&mut conn)
                        .map_err(|e| {
                            Status::internal(format!("Failed to delete event: {}", e))
                        })?;
        
                    Ok(tonic::Response::new(proto::RemoveEventResponse { success: true, message: "200 Ok".to_string() }))
                },
                None => Err(Status::not_found("Event not found")),
            }
        }
    
    async fn change_event(
        &self,
        request: tonic::Request<proto::ChangeEventRequest>
        ) -> Result<tonic::Response<proto::ChangeEventResponse>, Status> {
            let updated_event = request.get_ref().event.clone().ok_or_else(|| {
                Status::invalid_argument("Event is missing")
            })?;
        
            let events_list = CalendarService::load_events().await.map_err(|e| {
                Status::internal(format!("Failed to load events: {}", e))
            })?;
        
            let search_event = events_list.events.into_iter().find(|event| event.id == updated_event.id);
            
            match search_event {
                Some(_) => {        
                    let mut conn = get_connection();

                    diesel::update(events::events.filter(id.eq(updated_event.id as i32)))
                        .set(&Event {
                            id: updated_event.id as i32,
                            name: updated_event.name,
                            date: timestamp_to_system_time(updated_event.date),
                            fulltime: updated_event.fulltime,
                            author: updated_event.author,
                            create_at: timestamp_to_system_time(updated_event.create_at),
                            location: updated_event.location,
                            message: updated_event.message, 
                        })
                        .execute(&mut conn)
                        .map_err(|e| {
                            Status::internal(format!("Failed to update event: {}", e))
                        })?;
        
                    Ok(tonic::Response::new(proto::ChangeEventResponse { success: true, message: "201 Update".to_string() }))
                },
                None => Err(Status::not_found("Event not found")),
            }
        }
}

// ------------------------------------------------------------\\

/*
// ------------------------------------------------------------//
// Hello Service

#[derive(Debug, Default)]
struct HelloService {
    state: State,
}

impl HelloService {
    async fn counter(&self) {
        let mut count = self.state.write().await;
        *count += 1;
        println!("Requests count: {}", *count);
    }
}

#[tonic::async_trait]
impl Hello for HelloService {
    async fn send(
        &self,
        request: tonic::Request<proto::HelloRequest>
    ) -> Result<tonic::Response<proto::HelloResponse>, tonic::Status> {
        self.counter().await;

        let input = request.get_ref();

        if input.body.is_empty() || input.author.is_empty() || input.body.len() < 1 || input.author.len() < 1 {
            return Err(tonic::Status::invalid_argument("body or author is empty"));
        }

        let response = proto::HelloResponse {
            body: input.body.clone(),
            author: input.author.clone(),
        };

        Ok(tonic::Response::new(response))
    }
}

// ------------------------------------------------------------\\

// ------------------------------------------------------------//
// Admin Service

#[derive(Debug, Default)]
struct AdminService {
    state: State,
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

// ------------------------------------------------------------\\
*/


#[tokio::main]
async fn main() -> Result<(), Box<dyn  std::error::Error>> {
    dotenv().ok();

    let addr = "[::1]:50051".parse()?;

    /*let state = State::default();

    let hello = HelloService {
        state:  state.clone(),
    };
    let admin = AdminService{
        state: state.clone(),
    };*/
    let calendar = CalendarService;

    Server::builder()
        //.add_service(AdminServer::with_interceptor(admin, check_auth))
        //.add_service(HelloServer::new(hello))
        .add_service(CalendarServer::new(calendar))
        .serve(addr)
        .await?;
    
    Ok(())
}