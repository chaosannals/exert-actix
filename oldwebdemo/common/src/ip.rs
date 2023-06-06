use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
struct GetIpError {
    message: String,
}

impl GetIpError {
    fn new(message: &str) -> GetIpError {
        GetIpError {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for GetIpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for GetIpError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[allow(dead_code)]
pub fn get_wan_ip_sync() -> Result<String, Box<dyn Error>> {
    let response =
        reqwest::blocking::get("https://httpbin.org/ip")?.json::<HashMap<String, String>>()?;
    match response.get("origin") {
        Some(ip) => Ok(ip.clone()),
        None => Err(Box::new(GetIpError::new("get ip failed"))),
    }
}

pub async fn get_wan_ip() -> Result<String, Box<dyn Error>> {
    let response = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    match response.get("origin") {
        Some(ip) => Ok(ip.clone()),
        None => Err(Box::new(GetIpError::new("get ip failed"))),
    }
}
