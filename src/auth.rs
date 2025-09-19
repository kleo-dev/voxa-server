use std::sync::Arc;

use serde::Deserialize;

use crate::{Server, utils::client::Client};

#[derive(Debug, Deserialize)]
struct AuthApiRes {
    uuid: u32,
}

pub fn auth(_server: &Arc<Server>, client: &mut Client, token: &str) -> crate::Result<u32> {
    // let mut res = ureq::get(format!("https://api.voxa.org/server-auth?token={token}")).call()?;

    // let api_res: AuthApiRes = serde_json::from_str(&res.body_mut().read_to_string()?)?;

    client.set_uuid(232);

    Ok(232)
}
