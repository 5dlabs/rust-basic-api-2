mod app;
mod config;
mod error;
mod models;
mod repository;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let application = app::Application::build()?;
    application.run().await
}
