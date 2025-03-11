# gRPC server

This repository implements a gRPC server in Rust

## Contents

[Installation](#installation)\
[Client](#client)\
[Database](#database)\
[Changes](#changes)

## Installation

You will need to download Rust to run the project.

 * Download protocol buffers [github repository](https://github.com/protocolbuffers/protobuf)

> [!TIP]
> If you are using unix system: ```apt install -y protobuf-compiler```
> If you have problems or questions at this stage, read [installation guide](https://grpc.io/docs/protoc-installation/)

* Download Rust from [official site](https://www.rust-lang.org/ru/tools/install)
* Clone repository: ```git clone https://github.com/bjfssd757/gRPC_server.git```
* Run server: ```cargo run --bin server```

> [!NOTE]
> You can also run the client: ```cargo run --bin client```.\
>  Note that the client can only send a request to an enabled server!

## Client

The client code is implemented in the file [client.rs](src/client.rs)

In any case, to communicate with the server, you will need a gRPC client, since not all browsers are able to process responses from such servers. It is also important to have the same .proto files that are on the server.

> [!TIP]
> In fact, the functionality of "transferring" .proto files when accessing the server does exist. You can read about it on the Internet. At the moment (while you are seeing this insert), this project does not implement such functionality on the client.

## Database

> [!WARNING]
> At the moment, the database is not stable and may not start!  If you encounter such a problem, please report it in [issues](https://github.com/bjfssd757/gRPC_server/issues), mentioning your operating system and the error message!\
> If you do not want to use the database, you will need to delete the [.env file](.env), the [models.rs](src/models.rs), [schema.rs](src/schema.rs) and [diesel.toml](diesel.toml) files, and remove dependencies from [Cargo.toml](Cargo.toml) (not required, but recommended): r2d2, diesel, diesel_cli, r2d2-diesel. After that, you will need to rewrite the program logic in the [main.rs](src/main.rs) file in ```impl Calendar for CalendarService``` and ```impl CalendarService``` (lines 129 and 108 respectively).  You will probably also need to change the data types in structures (```struct```) and messages (```message```) in the proto files.
> Note that this will require knowledge of gRPC and especially Rust.
> This problem will be solved in the future!

In this example, Rust works with a PostgreSQL database using the [Diesel](https://diesel.rs/) library.

The [.env](.env) file specifies the path to your database:
```DATABASE_URL=postgres://your_profile_name:your_password@localhost/database_name```\
Where ```your_profile_name``` is your name in postgres;
```your_password``` is the postgres password;
```localhost``` is the address to connect to the database.  In this case, the database is on the same device. If necessary, replace with the IP address you need;\
```database_name``` - the name of the database you will work with.

Before starting the server, you need to create a PostgreSQL database.\
SQL shell (windows):

```sql
create database your_name;
\c your_name
create table table_name (smth);
```

## Changes

The project will evolve over time: bug fixes, new functionality added!\
Stay tuned!