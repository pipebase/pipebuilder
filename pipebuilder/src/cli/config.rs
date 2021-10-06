pub struct Config {
    // api endpoint
    pub endpoint: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            endpoint: String::from("127.0.0.1:16000"),
        }
    }
}
