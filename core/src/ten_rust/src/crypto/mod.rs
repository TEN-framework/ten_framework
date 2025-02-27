//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod bindings;

use aes::Aes128;
use anyhow::Result;
use ctr::cipher::{KeyIvInit, StreamCipher};
use serde::de::{self};
use serde::{Deserialize, Deserializer};

type Aes128Ctr64LE = ctr::Ctr128BE<Aes128>;

pub struct Aes128CtrParams {
    pub key: [u8; 16],
    pub nonce: [u8; 16],
}

pub struct Aes128CtrCipher {
    impl_cipher: Aes128Ctr64LE,
}

impl<'de> Deserialize<'de> for Aes128CtrParams {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawParams {
            key: String,
            nonce: String,
        }

        let raw = RawParams::deserialize(deserializer)?;

        let key: [u8; 16] = raw
            .key
            .as_bytes()
            .try_into()
            .map_err(|_| de::Error::custom("key must be 16 bytes"))?;

        let nonce: [u8; 16] = raw
            .nonce
            .as_bytes()
            .try_into()
            .map_err(|_| de::Error::custom("nonce must be 16 bytes"))?;

        Ok(Aes128CtrParams { key, nonce })
    }
}

pub enum EncryptionAlgorithm {
    Aes128Ctr(Aes128CtrCipher),
}

impl EncryptionAlgorithm {
    pub fn encrypt(&mut self, data: &mut [u8]) {
        match self {
            EncryptionAlgorithm::Aes128Ctr(cipher) => {
                cipher.impl_cipher.apply_keystream(data);
            }
        }
    }
}

pub fn create_encryption_algorithm(
    algorithm: &str,
    params_str: &str,
) -> Result<EncryptionAlgorithm> {
    match algorithm {
        "AES-CTR" => {
            let params: Aes128CtrParams = serde_json::from_str(params_str)?;
            let cipher =
                Aes128Ctr64LE::new(&params.key.into(), &params.nonce.into());
            Ok(EncryptionAlgorithm::Aes128Ctr(Aes128CtrCipher {
                impl_cipher: cipher,
            }))
        }
        _ => Err(anyhow::anyhow!("Invalid encryption algorithm")),
    }
}
