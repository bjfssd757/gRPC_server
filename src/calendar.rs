// Calendar service

#[macro_use]
extern crate lazy_static;
extern crate diesel;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use schema::events::id;
use std::env;
use proto::calendar_server::Calendar;
use tonic::Status;
use serde::{Deserialize, Serialize};
use self::models::Event;
use crate::schema::events::dsl as events;
use prost_types::Timestamp;

mod proto {
    tonic::include_proto!("calendar");
}
mod schema;
mod models;

pub type State = std::sync::Arc<tokio::sync::RwLock<u64>>;

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


#[derive(Serialize, Deserialize, Debug)]
pub struct EventProto {
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
pub struct EventsList {
    events: Vec<EventProto>,
}

#[derive(Debug, Default)]
pub struct CalendarService {
    state: State,
}

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
    async fn counter(&self) {
        let mut count = self.state.write().await;
        *count += 1;
    }
}

#[tonic::async_trait]
impl Calendar for CalendarService {
    async fn get_events(
        &self,
        _: tonic::Request<proto::GetEventsRequest>,
        ) -> Result<tonic::Response<proto::GetEventsResponse>, Status> {
        self.counter().await;

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
            self.counter().await;

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
            self.counter().await;

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
            self.counter().await;

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
            self.counter().await;

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