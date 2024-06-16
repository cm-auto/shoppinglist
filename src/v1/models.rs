use actix_web::error::UrlGenerationError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::prelude::FromRow;

#[derive(Serialize)]
pub(super) struct RestResource<'a, T: 'a + Serialize> {
    #[serde(flatten)]
    pub resource: &'a T,
    #[serde(rename = "_links")]
    pub links: Vec<String>,

    #[serde(rename = "_sub_resources", skip_serializing_if = "Option::is_none")]
    pub sub_resources: Option<Vec<String>>,
}

#[derive(Serialize, Clone, Debug)]
pub(super) struct User {
    pub id: i64,
    pub username: String,
    pub display_name: String,
}

impl User {
    pub fn rest_resource(
        &self,
        request: &actix_web::HttpRequest,
    ) -> Result<RestResource<User>, UrlGenerationError> {
        let id_string_array = [self.id.to_string()];

        let self_resource_name = resource_name!("/users/{identifier}");
        let self_id_url = request
            .url_for(self_resource_name, &id_string_array)
            .inspect_err(|_| {
                log::error!(
                    "Failed to get url for resource name: {}",
                    self_resource_name,
                );
            })?;
        let self_username_url =
            // since the same resource name is used, this won't be logged
            request.url_for(self_resource_name, [&self.username])?;

        let groups_resource_name = resource_name!("/users/{identifier}/groups");
        let groups_id_url = request
            .url_for(groups_resource_name, &id_string_array)
            .inspect_err(|_| {
                log::error!(
                    "Failed to get url for resource name: {}",
                    groups_resource_name,
                )
            })?;
        let groups_username_url = request.url_for(groups_resource_name, [&self.username])?;
        let sub_resources = Some(vec![
            groups_id_url.to_string(),
            groups_username_url.to_string(),
        ]);

        Ok(RestResource {
            resource: self,
            links: vec![self_id_url.to_string(), self_username_url.to_string()],
            sub_resources,
        })
    }
}

#[derive(Serialize)]
pub(super) struct Group {
    pub id: i64,
    pub name: String,
}

impl Group {
    pub fn rest_resource(
        &self,
        request: &actix_web::HttpRequest,
    ) -> Result<RestResource<Group>, UrlGenerationError> {
        let id_string_array = [self.id.to_string()];
        let self_resource_name = resource_name!("/groups/{id}");
        let self_id_url = request
            .url_for(self_resource_name, &id_string_array)
            .inspect_err(|_| {
                log::error!(
                    "Failed to get url for resource name: {}",
                    self_resource_name,
                );
            })?;

        let users_resource_name = resource_name!("/groups/{id}/users");
        let users_id_url = request
            .url_for(users_resource_name, &id_string_array)
            .inspect_err(|_| {
                log::error!(
                    "Failed to get url for resource name: {}",
                    users_resource_name,
                );
            })?;
        let sub_resources = Some(vec![users_id_url.to_string()]);

        Ok(RestResource {
            resource: self,
            links: vec![self_id_url.to_string()],
            sub_resources,
        })
    }
}

#[derive(Serialize, Clone, Debug, FromRow)]
pub(super) struct Entry {
    pub id: i64,
    pub product: String,
    pub amount: f32,
    pub unit: String,
    pub note: Option<String>,
    pub created: DateTime<Utc>,
    pub bought: Option<DateTime<Utc>>,
    pub user_id: i64,
    pub group_id: Option<i64>,
}

impl Entry {
    pub fn rest_resource(
        &self,
        request: &actix_web::HttpRequest,
    ) -> Result<RestResource<Entry>, UrlGenerationError> {
        let id_string_array = [self.id.to_string()];
        let self_resource_name = resource_name!("/entries/{id}");
        let self_id_url = request
            .url_for(self_resource_name, id_string_array)
            .inspect_err(|_| {
                log::error!(
                    "Failed to get url for resource name: {}",
                    self_resource_name,
                );
            })?;

        Ok(RestResource {
            resource: self,
            links: vec![self_id_url.to_string()],
            sub_resources: None,
        })
    }
}
