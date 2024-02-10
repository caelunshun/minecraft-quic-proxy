use minecraft_quic_proxy::gateway;
use quinn::{Endpoint, ServerConfig};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = cert.serialize_der()?;
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der)];

    let server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
    let endpoint = Endpoint::server(server_config, "0.0.0.0:6666".parse().unwrap())?;

    tracing::info!("Started");

    gateway::run(&endpoint, "temp").await?;

    Ok(())
}
