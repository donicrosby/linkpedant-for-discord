use linkpedant::Result;
use tracing;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .expect("could not set tracing global default");
    let config = linkpedant::get_configuration()?;
    let mut linkpedant = linkpedant::LinkPedant::new(config).await?;
    linkpedant.run().await
}
