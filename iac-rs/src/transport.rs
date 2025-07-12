use anyhow::{Context, Result, anyhow};
use quinn::Connection;
use quinn::Endpoint;
use quinn::{ClientConfig, ServerConfig};
use rcgen::{CertifiedKey, generate_simple_self_signed};
use rustls::client::danger::{
    DangerousClientConfigBuilder, HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};
use rustls::crypto::aws_lc_rs::default_provider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::{ServerName, UnixTime};
use rustls::server::ServerConfig as RustlsServerConfig;
use rustls::{ClientConfig as RustlsClientCfg, version::TLS13};
use rustls::{DigitallySignedStruct, SignatureScheme};
use std::sync::Arc;
use tokio::time::{Duration, timeout};

use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};

/// Generate a QUIC server config with a self-signed certificate
pub fn init_server() -> Result<ServerConfig> {
    let subject_alt_names = vec![
        "kevin-rs.dev".to_string(),
        "localhost".to_string(),
        "0.0.0.0".to_string(),
        "127.0.0.1".to_string(),
    ];
    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names)?;

    let cert_der = cert.der();
    let key_der = key_pair.serialize_der();
    let private_key = PrivateKeyDer::try_from(key_der)
        .map_err(|e| anyhow!("failed to parse private key: {e}"))?;

    let provider = Arc::new(default_provider());

    let tls_config = RustlsServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&TLS13])?
        .with_no_client_auth()
        .with_single_cert(vec![CertificateDer::from(cert_der.to_vec())], private_key)?;
    let tls_config = Arc::new(tls_config);

    let quic_tls_config = QuicServerConfig::try_from(tls_config.clone())?;

    Ok(ServerConfig::with_crypto(Arc::new(quic_tls_config)))
}

/// Generate a QUIC client config that accepts any certificate
/// TODO: Fix me
pub fn init_client() -> Result<ClientConfig> {
    let provider = Arc::new(default_provider());

    let base = RustlsClientCfg::builder_with_provider(provider)
        .with_protocol_versions(&[&TLS13])
        .expect("failed to set TLS versions");

    let dangerous = DangerousClientConfigBuilder { cfg: base };
    let tls_config = dangerous
        .with_custom_certificate_verifier(Arc::new(AllowAnyCert))
        .with_no_client_auth();
    let tls_config = Arc::new(tls_config);

    let quic_tls_config = QuicClientConfig::try_from(tls_config.clone())?;

    Ok(ClientConfig::new(Arc::new(quic_tls_config)))
}

#[derive(Debug)]
struct AllowAnyCert;

impl ServerCertVerifier for AllowAnyCert {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer,
        _intermediates: &[CertificateDer],
        _server_name: &ServerName,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
        ]
    }
}

pub async fn connect(addr: &str) -> Result<Connection> {
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(init_client()?);

    let connect_fut = endpoint.connect(addr.parse()?, "localhost")?;

    let conn = timeout(Duration::from_secs(5), connect_fut)
        .await
        .context("Connection attempt timed out")??;

    Ok(conn)
}
