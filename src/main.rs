use dotenv::dotenv;

pub mod app;
mod context;
mod errors;
mod pokemon;
mod pokemon_api;
mod storage;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    #[cfg(not(test))]
    app::init().await;
}
