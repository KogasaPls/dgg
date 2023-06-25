#![feature(lazy_cell)]

#[macro_use]
extern crate log;
extern crate dgg;

use config::Config;
use dgg::config::ChatAppConfig;
use dgg::dgg::chat::chat_client::ChatClient;
use std::sync::LazyLock;

static CONFIG: LazyLock<ChatAppConfig> = LazyLock::new(|| {
    let config = Config::builder()
        .add_source(config::Environment::default())
        .add_source(
            config::File::with_name("config")
                .required(true)
                .format(config::FileFormat::Toml),
        )
        .build()
        .expect("Failed to load config");

    ChatAppConfig::try_from(config).expect("Failed to load config")
});

fn init() {
    dotenv::dotenv().ok();
    pretty_env_logger::formatted_timed_builder()
        .parse_env("RUST_LOG")
        .init();

    info!("Starting...");
}

#[tokio::main]
async fn main() {
    init();

    let config = CONFIG.clone();
    let mut client = ChatClient::new(config);
    client.connect().await.expect("Failed to connect");

    while let Some(msg) = client.get_next_event().await.unwrap() {
        debug!("Received event: {:?}", msg);
    }

    debug!("Connected");
}
