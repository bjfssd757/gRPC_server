// @generated automatically by Diesel CLI.

diesel::table! {
    events (id) {
        id -> Int4,
        name -> Text,
        date -> Timestamp,
        fulltime -> Bool,
        author -> Text,
        create_at -> Timestamp,
        location -> Text,
        message -> Text,
    }
}
