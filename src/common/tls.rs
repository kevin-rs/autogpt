use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{crypto::aws_lc_rs::default_provider, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
    sync::Arc,
};
use tokio_rustls::TlsAcceptor;

/// Load certificates from a PEM file
pub fn load_certs<P: AsRef<Path>>(path: P) -> io::Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    certs(&mut reader).collect()
}

/// Load the first PKCS#8 private key from a PEM file
fn load_private_key<P: AsRef<Path>>(path: P) -> anyhow::Result<PrivateKeyDer<'static>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let keys: Result<Vec<_>, _> = pkcs8_private_keys(&mut reader).collect();
    let mut keys = keys?;
    if let Some(key) = keys.pop() {
        Ok(PrivateKeyDer::Pkcs8(key))
    } else {
        anyhow::bail!("No PKCS#8 private key found");
    }
}

/// Load TLS config and return a TlsAcceptor
pub fn load_tls_config() -> anyhow::Result<TlsAcceptor> {
    let cert_chain = load_certs("certs/cert.pem")?;
    let private_key = load_private_key("certs/key.pem")?;

    let provider = Arc::new(default_provider());

    let config = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}
