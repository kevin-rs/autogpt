use anyhow::{Result, anyhow};
use ed25519_compact::{KeyPair, PublicKey, Signature};
use tracing::{debug, error, instrument};

#[derive(Clone)]
pub struct Signer {
    keypair: KeyPair,
}

impl Signer {
    #[instrument(skip_all)]
    pub fn new(keypair: KeyPair) -> Self {
        debug!("🔐 Signer created");
        Self { keypair }
    }

    #[instrument(skip_all, fields(data_len = data.len()))]
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let signature = self.keypair.sk.sign(data, None);
        debug!(sig_len = signature.as_ref().len(), "✍️ Data signed");
        Ok(signature.to_vec())
    }

    pub fn verifying_key(&self) -> PublicKey {
        self.keypair.pk
    }
}

#[derive(Clone)]
pub struct Verifier {
    public_key: PublicKey,
}

impl Verifier {
    #[instrument(skip_all)]
    pub fn new(public_key: PublicKey) -> Self {
        debug!("🔍 Verifier initialized");
        Self { public_key }
    }

    #[instrument(skip_all, fields(data_len = data.len(), sig_len = sig.len()))]
    pub fn verify(&self, data: &[u8], sig: &[u8]) -> Result<()> {
        if sig.len() != 64 {
            error!("❌ Invalid signature length: {}", sig.len());
            return Err(anyhow!("Invalid signature length"));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(sig);
        let signature = Signature::new(sig_array);

        self.public_key.verify(data, &signature).map_err(|_| {
            error!("❌ Signature verification failed");
            anyhow!("Signature verification failed")
        })?;

        debug!("✅ Signature verified successfully");
        Ok(())
    }
}

#[instrument]
pub fn generate_key() -> KeyPair {
    debug!("🔑 Generating new keypair");
    KeyPair::generate()
}
