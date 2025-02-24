use crate::schema::events;
use diesel::prelude::*;
//use crate::schema::users;

/*/
#[derive(Queryable, Selectable, QueryableByName)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub role: String,
}
*/

#[derive(Queryable, Selectable, QueryableByName, Insertable, AsChangeset)]
#[table_name = "events"]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub date: std::time::SystemTime,
    pub fulltime: bool,
    pub author: String,
    pub create_at: std::time::SystemTime,
    pub location: String,
    pub message: String,
}