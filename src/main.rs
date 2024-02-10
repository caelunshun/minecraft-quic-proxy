use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use minecraft_quic_proxy::{gateway, gateway::AuthenticationKey};
use quinn::{Endpoint, ServerConfig};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Gateway(GatewayArgs),
}

#[derive(Debug, Args)]
struct GatewayArgs {
    #[arg(short, long, default_value = "6666")]
    port: u16,
    #[arg(long)]
    self_signed_cert: bool,
    #[arg(long)]
    cert: Option<PathBuf>,
    #[arg(long)]
    priv_key: Option<PathBuf>,
    #[arg(long)]
    auth_key: String,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let Command::Gateway(args) = cli.command;

    let server_config = if args.self_signed_cert {
        server_config_self_signed()?
    } else {
        server_config_with_cert(
            args.cert
                .as_ref()
                .context("must provide a certificate path or enable --self-signed-cert")?,
            args.priv_key
                .as_ref()
                .context("must provide a private key path")?,
        )?
    };

    let endpoint = Endpoint::server(
        server_config,
        format!("0.0.0.0:{}", args.port).parse().unwrap(),
    )?;

    let authentication_key = if argon2::PasswordHash::new(&args.auth_key).is_ok() {
        AuthenticationKey::Hashed(args.auth_key)
    } else {
        tracing::warn!("Using plaintext authentication key. This is likely to expose side channel vulnerabilities.");
        AuthenticationKey::Plaintext(args.auth_key)
    };

    tracing::info!("Listening on {}", endpoint.local_addr()?);
    gateway::run(&endpoint, &authentication_key).await?;

    Ok(())
}

fn server_config_with_cert(cert_path: &Path, priv_key_path: &Path) -> anyhow::Result<ServerConfig> {
    // Code adapted from Quinn examples
    let key = fs_err::read(priv_key_path).context("failed to read private key")?;
    let mut key = key.as_slice();
    let key = if priv_key_path.extension().map_or(false, |x| x == "der") {
        rustls::PrivateKey(key.to_vec())
    } else {
        let mut pkcs8 = rustls_pemfile::pkcs8_private_keys(&mut key);
        match pkcs8.next() {
            Some(x) => rustls::PrivateKey(x?.secret_pkcs8_der().to_vec()),
            None => {
                drop(pkcs8);
                let rsa = rustls_pemfile::rsa_private_keys(&mut key);
                match rsa.into_iter().next() {
                    Some(x) => rustls::PrivateKey(x?.secret_pkcs1_der().to_vec()),
                    None => {
                        anyhow::bail!("no private keys found");
                    }
                }
            }
        }
    };
    let cert_chain = fs_err::read(cert_path).context("failed to read certificate chain")?;
    let cert_chain = if cert_path.extension().map_or(false, |x| x == "der") {
        vec![rustls::Certificate(cert_chain)]
    } else {
        rustls_pemfile::certs(&mut &*cert_chain)
            .into_iter()
            .map(|cert| cert.map(|der| rustls::Certificate(der.to_vec())))
            .collect::<Result<Vec<_>, std::io::Error>>()?
    };

    Ok(quinn::ServerConfig::with_single_cert(cert_chain, key)?)
}

fn server_config_self_signed() -> anyhow::Result<ServerConfig> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = cert.serialize_der()?;
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der)];

    Ok(ServerConfig::with_single_cert(cert_chain, priv_key)?)
}
