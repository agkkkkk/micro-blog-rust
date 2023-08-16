// @generated automatically by Diesel CLI.

diesel::table! {
    likes (id) {
        id -> Uuid,
        created_at -> Timestamp,
        blog_id -> Uuid,
    }
}

diesel::table! {
    microblogs (id) {
        id -> Uuid,
        blog_message -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (email) {
        #[max_length = 50]
        email -> Varchar,
        #[max_length = 30]
        username -> Varchar,
        #[max_length = 10]
        dateofbirth -> Nullable<Varchar>,
        #[max_length = 10]
        contact -> Nullable<Varchar>,
        password -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    likes,
    microblogs,
    users,
);
