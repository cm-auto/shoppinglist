use actix_web::{
    http::Method,
    web::{self, ServiceConfig},
};

use super::handlers::*;

pub fn configure_routes(config: &mut ServiceConfig) {
    let index_resource = web::resource("")
        .name(resource_name!(""))
        .get(get_api_index)
        .head(get_api_index)
        .route(generate_options_route!("GET, HEAD, OPTIONS"));
    config.service(index_resource);

    let user_by_identifier_resource = web::resource("/users/{identifier}")
        .name(resource_name!("/users/{identifier}"))
        .get(get_user_by_id_or_username)
        // actix-web makes head response' body automatically empty, so get can be used as its handler
        .head(get_user_by_id_or_username)
        .route(generate_options_route!("GET, HEAD, OPTIONS"));
    config.service(user_by_identifier_resource);

    let user_groups_resource = web::resource("/users/{identifier}/groups")
        .name(resource_name!("/users/{identifier}/groups"))
        .get(get_user_groups_by_id_or_username)
        .head(get_user_groups_by_id_or_username)
        .route(generate_options_route!("GET, HEAD, OPTIONS"));
    config.service(user_groups_resource);

    let groups_resource = web::resource("/groups")
        .name(resource_name!("/groups"))
        .get(get_groups)
        .head(get_groups)
        .route(generate_options_route!("GET, HEAD, OPTIONS"));
    config.service(groups_resource);

    let group_by_id_resource = web::resource("/groups/{id}")
        .name(resource_name!("/groups/{id}"))
        .get(get_group_by_id)
        .head(get_group_by_id)
        .route(generate_options_route!("GET, HEAD, OPTIONS"));
    config.service(group_by_id_resource);

    let group_users_resource = web::resource("/groups/{id}/users")
        .name(resource_name!("/groups/{id}/users"))
        .get(get_group_users)
        .head(get_group_users)
        .route(generate_options_route!("GET, HEAD, OPTIONS"));
    config.service(group_users_resource);

    let entries_resource = web::resource("/entries")
        .name(resource_name!("/entries"))
        .get(get_entries)
        .head(get_entries)
        .post(post_entry)
        // TODO: is there a way to auto generate this when using Resource?
        // it seems like it automatically generates the allow header
        // if the client requests using a method that is not implemented...
        .route(generate_options_route!("GET, HEAD, POST, OPTIONS"));
    config.service(entries_resource);

    let entries_by_id_resource = web::resource("/entries/{id}")
        .name(resource_name!("/entries/{id}"))
        .get(get_entry_by_id)
        .head(get_entry_by_id)
        .patch(patch_entry)
        .delete(delete_entry)
        .route(generate_options_route!("GET, HEAD, PATCH, DELETE, OPTIONS"));
    config.service(entries_by_id_resource);
}
