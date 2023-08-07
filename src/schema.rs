// @generated automatically by Diesel CLI.

use diesel::prelude::*;

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

joinable!(likes -> microblogs(blog_id));

diesel::allow_tables_to_appear_in_same_query!(likes, microblogs,);
