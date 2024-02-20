// @generated automatically by Diesel CLI.

diesel::table! {
    album (id) {
        id -> Uuid,
        title -> Text,
        description -> Nullable<Text>,
        original_title -> Text,
        is_uploaded -> Bool,
        prev_image_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    album_image (id) {
        id -> Uuid,
        album_id -> Uuid,
        image_id -> Uuid,
        order_index -> Int4,
        highlighted -> Bool,
        is_primary_album -> Bool,
    }
}

diesel::table! {
    image (id) {
        id -> Uuid,
        title -> Text,
        original_full_title -> Text,
        description -> Nullable<Text>,
        path -> Text,
        width -> Nullable<Int4>,
        height -> Nullable<Int4>,
        is_uploaded -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        hash -> Varchar,
        is_admin -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(album -> image (prev_image_id));
diesel::joinable!(album_image -> album (album_id));
diesel::joinable!(album_image -> image (image_id));

diesel::allow_tables_to_appear_in_same_query!(
    album,
    album_image,
    image,
    users,
);
