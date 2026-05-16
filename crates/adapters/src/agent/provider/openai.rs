use rig::providers::openai;
pub fn client(api_key: &str, base_url: &str) -> Result<openai::Client, String> {
    openai::Client::builder()
        .api_key(api_key)
        .base_url(base_url)
        .build()
        .map_err(|err| err.to_string())
}
