use namada_core::types::key::common;
use namada_core::types::key::ed25519;
use namada_core::types::key::SecretKey;
use namada_core::types::key::SigScheme;

pub fn random_key() -> common::SecretKey {
    let mut rng = rand::thread_rng();
    ed25519::SigScheme::generate(&mut rng).try_to_sk().unwrap()
}
