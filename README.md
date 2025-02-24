# gRPC server

Этот репозиторий реализует gRPC сервер на языке Rust

## Оглавление

[Установка](#установка)\
[Клиент](#клиент)\
[База данных](#база-данных)\
[Изменения](#изменения)

## Установка

Для запуска проекта вам потребуется скачать Rust.

* Качаем protocol buffers [github репозиторий](https://github.com/protocolbuffers/protobuf)

> [!TIP]
> Если вы используете unix систему: ```apt install -y protobuf-compiler```
> Если у вас возникли проблемы или вопросы на этом этапе, прочитайте [гайд по установке](https://grpc.io/docs/protoc-installation/)

* Качаем Rust с [официального сайта](https://www.rust-lang.org/ru/tools/install)
* Клонируем репозиторий: ```git clone https://github.com/bjfssd757/gRPC_server.git```
* Запускаем сервер: ```cargo run --bin server```

> [!NOTE]
> Также вы можете запустить клиента: ```cargo run --bin client```.\
> Учтите, что клиент может отправить запрос только на включённый сервер!

## Клиент

Код клиента реализован в файле [client.rs](src/client.rs)

В любом случае для связи с сервером вам потребуется gRPC клиент, т.к. не все браузеры способны обрабатывать ответы от таких серверов. Также важно иметь такие же .proto файлы, какие есть на сервере.

> [!TIP]
> На самом деле функционал "передачи" .proto файлов при обращении на сервер есть. О нём можно почитать в Интернете. На данный момент (пока вы видите эту вставку) данный проект не реализует такого функционала на клиенте.

## База данных

> [!WARNING]
> На данный момент база данных работает не стабильно и может не запуститься! Если вы столкнули с такой проблемой, то большая просьба сообщить о ней в [issues](https://github.com/bjfssd757/gRPC_server/issues), упоминув свою операционную систему и сообщение об ошибке!\
> Если вы не хотите использовать базу данных, то вам потребуется удалить [файл .env](.env), файлы [models.rs](src/models.rs), [schema.rs](src/schema.rs) и [diesel.toml](diesel.toml), а также удалить зависимости из [Cargo.toml](Cargo.toml) (не обязательно, но желательно): r2d2, diesel, diesel_cli, r2d2-diesel. После этого вам потребуется переписать логику программы в файле [main.rs](src/main.rs) в ```impl Calendar for CalendarService``` и ```impl CalendarService``` (129 и 108 строки соответственно). Вероятно, что вам также потребуется изменить типы данных в структурах (```struct```) и сообщения (```message```) в proto-файлах\
> Учтите, что для этого вам потребуются знания gRPC и в особенности языка Rust.\
> В будущем эта проблема будет решена!

В данном примере Rust работает с базой данных PostgreSQL с помощью библиотеки [Diesel](https://diesel.rs/).

В файле [.env](.env) указывается путь до вашей базы данных:
```DATABASE_URL=postgres://ваше_имя_профиля:ваш_пароль@localhost/название_базы_данных```\
Где ```ваше_имя_профиля``` - ваше имя в postgres;\
```ваш_пароль``` - пароль postgres;\
```localhost``` - адрес для подключения к базе. В данном случае база находится на этом же устройстве. Если потребуется, замените на нужный вам IP адрес;\
```название_базы_данных``` - название базы данных, с которой вы будете работать.

Перед запуском сервера вам необходимо создать базу данных PostgreSQL.\
SQL shell (windows):

```sql
create database your_name;
\c your_name
create table table_name (smth);
```

## Изменения

Проект будет развиваться со временем: исправление багов, добавление нового функционала!\
Следите за обновлениями!
