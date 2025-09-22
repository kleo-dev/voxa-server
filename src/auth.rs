use std::sync::Arc;

use serde::Deserialize;

use crate::{Server, utils::client::Client};

#[derive(Debug, Deserialize)]
struct AuthApiRes {
    user_id: u32,
}

pub fn auth(_server: &Arc<Server>, client: &mut Client, token: &str) -> crate::Result<u32> {
    let mut res = ureq::get(format!(
        "http://localhost:3000/api/auth?intents=server&token={token}"
    ))
    .call()?;
    let api_res: AuthApiRes = serde_json::from_str(&res.body_mut().read_to_string()?)?;
    client.set_uuid(api_res.user_id);
    Ok(api_res.user_id)
}
