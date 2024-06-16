use std::collections::BTreeMap;

use actix_web::{
    http::StatusCode,
    web::{self, Json, ReqData},
    HttpResponse, HttpResponseBuilder,
};
use is_empty::IsEmpty;
use serde::{Deserialize, Deserializer};
use sqlx::{Pool, Postgres, QueryBuilder};

use crate::{v1::models::Entry, AppData};

use super::models::{Group, User};

macro_rules! url_for_static_or_return {
    ($request: expr, $name: expr) => {
        match $request.url_for_static($name) {
            Ok(url) => url,
            Err(_) => {
                log::error!("Failed to get static url for resource name: {}", $name);
                return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .json("internal server error");
            }
        }
    };
}

// responds with a json that shows all available resources and their urls
pub async fn get_api_index(request: actix_web::HttpRequest) -> impl actix_web::Responder {
    // a BTreeMap is used to keep the order of the keys
    let body = BTreeMap::from([
        (
            "entries",
            url_for_static_or_return!(&request, resource_name!("/entries")).to_string(),
        ),
        (
            "groups",
            url_for_static_or_return!(&request, resource_name!("/groups")).to_string(),
        ),
        // was used for testing
        // (
        //     "users",
        //     url_for_static_or_return!(&request, resource_name!("/users")).to_string(),
        // ),
    ]);

    HttpResponseBuilder::new(StatusCode::OK).json(body)
}

// if there is a newer version of the api (e.g. "v2"),
// these two macros should be moved to another module
macro_rules! ok_or_log_and_respond_internal_server_error {
    ($result: expr) => {
        match $result {
            Ok(res) => res,
            Err(err) => {
                log::error!("Internal server error: {}", err);
                return HttpResponse::InternalServerError().json("internal server error");
            }
        }
    };
}

macro_rules! all_ok_or_log_and_respond_internal_server_error {
    ($result_vec: expr) => {{
        let mut vector = Vec::with_capacity($result_vec.len());
        for result in $result_vec {
            vector.push(ok_or_log_and_respond_internal_server_error!(result));
        }
        vector
    }};
}

// was used for testing
// pub async fn get_users(
//     request: actix_web::HttpRequest,
//     app_data: web::Data<AppData>,
// ) -> HttpResponse {
//     let pool = &app_data.pool;
//     let users_result = sqlx::query_as!(
//         User,
//         r#"select id, username, display_name from users order by id"#
//     )
//     .fetch_all(pool)
//     .await;
//     let users = ok_or_log_and_respond_internal_server_error!(users_result);
//
//     let rest_resources = all_ok_or_log_and_respond_internal_server_error!(users
//         .iter()
//         .map(|user| user.rest_resource(&request))
//         .collect::<Vec<_>>());
//
//     HttpResponse::Ok().json(rest_resources)
// }

pub async fn get_user_by_id_or_username(
    request: actix_web::HttpRequest,
    identifier: web::Path<String>,
    app_data: web::Data<AppData>,
    user_id: ReqData<i64>,
) -> HttpResponse {
    let user_id = user_id.into_inner();
    let identifier_parsed_option = identifier.parse::<i64>().ok();
    // if the identifier is a number we can check if the user has access before
    // querying the database
    if let Some(identifier_parsed) = identifier_parsed_option {
        if user_id != identifier_parsed {
            return HttpResponse::NotFound().json("user not found");
        }
    }
    let user_option = ok_or_log_and_respond_internal_server_error!(
        sqlx::query_as!(
            User,
            r#"select id, username, display_name from users where (username = $1 or id = $2) and id = $3"#,
            identifier.as_str(),
            identifier_parsed_option,
            user_id, // if the username was passed as identifier, this check makes sure that the user can't see other users
        )
        .fetch_optional(&app_data.pool)
        .await
    );
    let Some(user) = user_option else {
        return HttpResponse::NotFound().json("user not found");
    };

    // why does rest_resource log?
    // because it has access to the resource_name
    // this is why the ok_or_log_and_respond_internal_server_error macro is not used
    // let rest_resource = ok_or_log_and_respond_internal_server_error!(user.rest_resource(&request));
    let Ok(rest_resource) = user.rest_resource(&request) else {
        return HttpResponse::InternalServerError().json("internal server error");
    };

    HttpResponse::Ok().json(rest_resource)
}

pub async fn get_groups(
    request: actix_web::HttpRequest,
    app_data: web::Data<AppData>,
    user_id: ReqData<i64>,
) -> HttpResponse {
    let user_id = user_id.into_inner();
    let groups = ok_or_log_and_respond_internal_server_error!(
        sqlx::query_as!(Group, r#"select id, name from groups inner join users_groups_relations as ugr on ugr.group_id = id and ugr.user_id = $1 order by id"#, user_id)
            .fetch_all(&app_data.pool)
            .await
    );
    let rest_resources = all_ok_or_log_and_respond_internal_server_error!(groups
        .iter()
        .map(|item| item.rest_resource(&request))
        .collect::<Vec<_>>());

    HttpResponse::Ok().json(rest_resources)
}

pub async fn get_group_by_id(
    request: actix_web::HttpRequest,
    id: web::Path<i64>,
    app_data: web::Data<AppData>,
    user_id: ReqData<i64>,
) -> HttpResponse {
    let user_id = user_id.into_inner();
    let resource_option = ok_or_log_and_respond_internal_server_error!(
        sqlx::query_as!(
            Group,
            r#"select id, name from groups inner join users_groups_relations as ugr on ugr.group_id = id and ugr.user_id = $1 where id = $2"#,
            user_id,
            id.into_inner(),
        )
        .fetch_optional(&app_data.pool)
        .await
    );

    let Some(resource) = resource_option else {
        return HttpResponse::NotFound().json("group not found");
    };

    let rest_resource =
        ok_or_log_and_respond_internal_server_error!(resource.rest_resource(&request));

    HttpResponse::Ok().json(rest_resource)
}

pub async fn get_group_users(
    request: actix_web::HttpRequest,
    id: web::Path<i64>,
    app_data: web::Data<AppData>,
    user_id: ReqData<i64>,
) -> HttpResponse {
    let group_id = id.into_inner();
    let user_id = user_id.into_inner();
    let is_member = ok_or_log_and_respond_internal_server_error!(
        is_member(&app_data.pool, user_id, group_id).await
    );
    if !is_member {
        return HttpResponse::NotFound().json("group not found");
    }
    let users = ok_or_log_and_respond_internal_server_error!(sqlx::query_as!(
        User,
        r#"select id, username, display_name from users inner join users_groups_relations as ugr on users.id = ugr.user_id where ugr.group_id = $1"#,
        group_id,
    ).fetch_all(&app_data.pool).await);
    let body = all_ok_or_log_and_respond_internal_server_error!(users
        .iter()
        .map(|user| user.rest_resource(&request))
        .collect::<Vec<_>>());

    HttpResponse::Ok().json(body)
}

pub async fn get_user_groups_by_id_or_username(
    request: actix_web::HttpRequest,
    identifier: web::Path<String>,
    app_data: web::Data<AppData>,
    user_id: ReqData<i64>,
) -> HttpResponse {
    let identifer_string = identifier.into_inner();
    let user_id = user_id.into_inner();
    let groups = ok_or_log_and_respond_internal_server_error!(sqlx::query_as!(
        Group,
        r#"select id, name from groups inner join users_groups_relations as ugr on groups.id = ugr.group_id and ugr.user_id = $1 where ugr.user_id = $2 or ugr.user_id in (select id from users where username = $3)"#,
        user_id,
        identifer_string.parse::<i64>().ok(),
        identifer_string
    ).fetch_all(&app_data.pool).await);

    let body = all_ok_or_log_and_respond_internal_server_error!(groups
        .iter()
        .map(|resource| resource.rest_resource(&request))
        .collect::<Vec<_>>());

    HttpResponse::Ok().json(body)
}

pub async fn get_entries(
    request: actix_web::HttpRequest,
    app_data: web::Data<AppData>,
    user_id: ReqData<i64>,
) -> HttpResponse {
    let pool = &app_data.pool;
    let rows_result = sqlx::query_as!(
        Entry,
        // only show entries of groups the user is a member of
        // or their personal entries
        // this does not show entries that were created by the user
        // but the user is not part of the assigned group anymore
        // this is intentional!
        r#"select
            e.id, e.product, e.amount, e.unit, e.note, e.created, e.bought, e.user_id, e.group_id
            from
                entries as e
            left outer join
                users_groups_relations as ugr
                    on ugr.group_id = e.group_id
                    and ugr.user_id = $1
            where
                ugr.group_id is null and e.user_id = $1
                or ugr.group_id is not null
            order by e.id"#,
        user_id.into_inner(),
    )
    .fetch_all(pool)
    .await;
    let rows = ok_or_log_and_respond_internal_server_error!(rows_result);

    let rest_resources = all_ok_or_log_and_respond_internal_server_error!(rows
        .iter()
        .map(|user| user.rest_resource(&request))
        .collect::<Vec<_>>());

    HttpResponse::Ok().json(rest_resources)
}

async fn is_member(
    pool: &Pool<Postgres>,
    user_id: i64,
    group_id: i64,
) -> Result<bool, sqlx::Error> {
    let is_member_result = sqlx::query!(
        r#"select exists (select 1 from users_groups_relations where user_id = $1 and group_id = $2) as "exists!: bool""#,
        user_id,
        group_id
    )
    .fetch_one(pool)
    .await;
    Ok(is_member_result?.exists)
}

#[derive(Deserialize)]
pub(super) struct PostEntryRequestData {
    product: String,
    amount: f32,
    unit: String,
    note: Option<String>,
    group_id: Option<i64>,
}

pub(super) async fn post_entry(
    request: actix_web::HttpRequest,
    app_data: web::Data<AppData>,
    user_id: ReqData<i64>,
    payload: Json<PostEntryRequestData>,
) -> HttpResponse {
    let user_id = user_id.into_inner();
    let pool = &app_data.pool;
    if let Some(group_id) = payload.group_id {
        let is_member =
            ok_or_log_and_respond_internal_server_error!(is_member(pool, user_id, group_id).await);
        if !is_member {
            return HttpResponse::NotFound().json("group not found");
        }
    }
    let row_result = sqlx::query_as!(
        Entry,
        r#"insert into entries (product, amount, unit, note, user_id, group_id)
            values ($1, $2, $3, $4, $5, $6)
        returning id, product, amount, unit, note, user_id, group_id, created, bought"#,
        payload.product,
        payload.amount,
        payload.unit,
        payload.note,
        user_id,
        payload.group_id,
    )
    .fetch_one(pool)
    .await;
    let row = ok_or_log_and_respond_internal_server_error!(row_result);

    let rest_resource = ok_or_log_and_respond_internal_server_error!(row.rest_resource(&request));

    HttpResponse::Created().json(rest_resource)
}

fn deserialize_option_string<'de, D>(input: D) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let string_option = Option::<String>::deserialize(input)?;
    Ok(Some(string_option))
}

#[derive(Deserialize, IsEmpty)]
pub(super) struct PatchEntryRequestData {
    product: Option<String>,
    amount: Option<f32>,
    unit: Option<String>,
    // the first option shows if a value has been supplied at all
    // the second option shows if a text or null has been supplied
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_option_string")]
    note: Option<Option<String>>,
    bought: Option<bool>,
}

async fn can_modify_entry(
    pool: &Pool<Postgres>,
    user_id: i64,
    entry_id: i64,
) -> Result<bool, sqlx::Error> {
    let entry_option = sqlx::query!(
        r#"select entries.user_id, entries.group_id, case when ugr.group_id is null then false else true end as "is_member!: bool" from entries left outer join users_groups_relations as ugr on entries.group_id = ugr.group_id and ugr.user_id = $1 where entries.id = $2"#,
        user_id,
        entry_id
    )
    .fetch_optional(pool)
    .await?;
    let entry = match entry_option {
        None => return Ok(false),
        Some(value) => value,
    };
    if entry.group_id.is_none() && entry.user_id != user_id
        || entry.group_id.is_some() && !entry.is_member
    {
        return Ok(false);
    }
    Ok(true)
}

// currently read access and modify access have the same requirements
// if this later changes the functions that use "can_read_entry" will not need to be changed
// additionally using "can_read_entry" make it more clear, that we intend to only read an entry
#[inline(always)]
async fn can_read_entry(
    pool: &Pool<Postgres>,
    user_id: i64,
    entry_id: i64,
) -> Result<bool, sqlx::Error> {
    can_modify_entry(pool, user_id, entry_id).await
}

pub(super) async fn patch_entry(
    request: actix_web::HttpRequest,
    app_data: web::Data<AppData>,
    entry_id: web::Path<i64>,
    user_id: ReqData<i64>,
    payload: Json<PatchEntryRequestData>,
) -> HttpResponse {
    let payload = payload.0;
    if payload.is_empty() {
        return HttpResponse::BadRequest().json("specify at least one field!");
    }
    let entry_id = entry_id.into_inner();
    let user_id = user_id.into_inner();
    let pool = &app_data.pool;
    let can_modify_entry = ok_or_log_and_respond_internal_server_error!(
        can_modify_entry(pool, user_id, entry_id).await
    );
    if !can_modify_entry {
        return HttpResponse::NotFound().json("entry not found");
    }
    let mut query_builder = QueryBuilder::<Postgres>::new("update entries set ");
    if let Some(product) = payload.product {
        query_builder.push(" product = ");
        query_builder.push_bind(product);
    }
    if let Some(value) = payload.amount {
        query_builder.push(" amount = ");
        query_builder.push_bind(value);
    }
    if let Some(value) = payload.unit {
        query_builder.push(" unit = ");
        query_builder.push_bind(value);
    }
    if let Some(value) = payload.note {
        query_builder.push(" note = ");
        query_builder.push_bind(value);
    }
    if let Some(value) = payload.bought {
        query_builder.push(" bought = ");
        if value {
            query_builder.push("now()");
        } else {
            query_builder.push("null");
        }
    }

    query_builder.push(" where id = ");
    query_builder.push_bind(entry_id);
    query_builder
        .push(" returning id, product, amount, unit, note, user_id, group_id, created, bought");

    let query = query_builder.build_query_as::<Entry>();
    let entry_result = query.fetch_optional(pool).await;
    let entry_option = ok_or_log_and_respond_internal_server_error!(entry_result);
    let entry = match entry_option {
        Some(entry) => entry,
        None => {
            return HttpResponse::NotFound().json("entry not found");
        }
    };

    let rest_resource = ok_or_log_and_respond_internal_server_error!(entry.rest_resource(&request));
    HttpResponse::Ok().json(rest_resource)
}

pub(super) async fn delete_entry(
    app_data: web::Data<AppData>,
    entry_id: web::Path<i64>,
    user_id: ReqData<i64>,
) -> HttpResponse {
    let entry_id = entry_id.into_inner();
    let user_id = user_id.into_inner();
    let pool = &app_data.pool;
    let can_modify_entry = ok_or_log_and_respond_internal_server_error!(
        can_modify_entry(pool, user_id, entry_id).await
    );
    if !can_modify_entry {
        return HttpResponse::NotFound().json("entry not found");
    }

    ok_or_log_and_respond_internal_server_error!(
        sqlx::query!("delete from entries where id = $1", entry_id)
            .execute(pool)
            .await
    );

    HttpResponse::NoContent().finish()
}

pub(super) async fn get_entry_by_id(
    request: actix_web::HttpRequest,
    app_data: web::Data<AppData>,
    entry_id: web::Path<i64>,
    user_id: ReqData<i64>,
) -> HttpResponse {
    let entry_id = entry_id.into_inner();
    let user_id = user_id.into_inner();
    let pool = &app_data.pool;
    let can_read_entry =
        ok_or_log_and_respond_internal_server_error!(can_read_entry(pool, user_id, entry_id).await);
    if !can_read_entry {
        return HttpResponse::NotFound().json("entry not found");
    }

    let row_result = sqlx::query_as!(
        Entry,
        r#"select
            e.id, e.product, e.amount, e.unit, e.note, e.created, e.bought, e.user_id, e.group_id
            from
                entries as e
            left outer join
                users_groups_relations as ugr
                    on ugr.group_id = e.group_id
                    and ugr.user_id = $1
            where
                (ugr.group_id is null and e.user_id = $1
                or ugr.group_id is not null)
                and e.id = $2
            order by e.id"#,
        user_id,
        entry_id,
    )
    .fetch_optional(pool)
    .await;
    let row_option = ok_or_log_and_respond_internal_server_error!(row_result);
    let row = match row_option {
        Some(value) => value,
        None => return HttpResponse::NotFound().json("entry not found"),
    };

    let rest_resource = ok_or_log_and_respond_internal_server_error!(row.rest_resource(&request));

    HttpResponse::Created().json(rest_resource)
}
