table! {
    blocks (source, target) {
        source -> BigInt,
        target -> BigInt,
        retrieved_at -> BigInt,
    }
}

table! {
    credentials (id) {
        id -> Integer,
        identifier -> Text,
        secret -> Text,
    }
}

table! {
    default_user (id) {
        id -> Integer,
        user -> BigInt,
    }
}

table! {
    endpoints (id) {
        id -> Integer,
        uri -> Text,
    }
}

table! {
    tokens (id) {
        id -> Integer,
        client -> Integer,
        token -> Integer,
        user -> BigInt,
    }
}

table! {
    user_list_cursors (endpoint, authenticated_user, user) {
        endpoint -> Integer,
        authenticated_user -> BigInt,
        user -> BigInt,
        cursor -> BigInt,
    }
}

table! {
    users (id) {
        id -> BigInt,
    }
}

joinable!(default_user -> users (user));
joinable!(tokens -> users (user));
joinable!(user_list_cursors -> endpoints (endpoint));

allow_tables_to_appear_in_same_query!(
    blocks,
    credentials,
    default_user,
    endpoints,
    tokens,
    user_list_cursors,
    users,
);
