use reqwest::Client;

pub fn new_client() -> reqwest::Result<Client> {
    const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    trace!("user agent name: {}", USER_AGENT);
    Client::builder().user_agent(USER_AGENT).build()
}
