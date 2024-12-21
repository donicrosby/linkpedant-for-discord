#[tokio::main]
async fn main() -> linkpedant::Result<()> {
    let subscriber = linkpedant::get_subscriber("info".into());
    linkpedant::init_subscriber(subscriber);
    let config = linkpedant::get_configuration()?;
    let mut linkpedant = linkpedant::LinkPedant::new(config).await?;
    linkpedant.run().await
}
