use crate::config::BasicAuth;
use pipebuilder_common::Result;
use reqwest::{
    header::{HeaderMap, HeaderName},
    Body, Client, Response,
};
use serde::Serialize;
use std::collections::HashMap;

pub async fn do_post<B>(
    body: B,
    url: &str,
    headers: HashMap<String, String>,
    basic_auth: Option<BasicAuth>,
    bearer_auth_token: Option<String>,
) -> Result<Response>
where
    B: Into<Body>,
{
    let mut hmap = HeaderMap::new();
    for (name, value) in headers {
        hmap.insert::<HeaderName>(name.parse()?, value.parse()?);
    }
    let request = Client::new().post(url).headers(hmap);
    let request = match basic_auth {
        Some(ref basic_auth) => {
            request.basic_auth(&basic_auth.username, basic_auth.password.as_ref())
        }
        None => request,
    };
    let request = match bearer_auth_token {
        Some(ref token) => request.bearer_auth(token),
        None => request,
    };
    let response = request.body(body).send().await?;
    Ok(response)
}

pub async fn do_query<Q>(
    url: &str,
    headers: HashMap<String, String>,
    query: Q,
    basic_auth: Option<BasicAuth>,
    bearer_auth_token: Option<String>,
) -> Result<Response>
where
    Q: Serialize,
{
    let mut hmap = HeaderMap::new();
    for (name, value) in headers {
        hmap.insert::<HeaderName>(name.parse()?, value.parse()?);
    }
    let request = Client::new().get(url).headers(hmap).query(&query);
    let request = match basic_auth {
        Some(ref basic_auth) => {
            request.basic_auth(&basic_auth.username, basic_auth.password.as_ref())
        }
        None => request,
    };
    let request = match bearer_auth_token {
        Some(ref token) => request.bearer_auth(token),
        None => request,
    };
    let response = request.send().await?;
    Ok(response)
}
