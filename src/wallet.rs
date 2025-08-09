use bitcoin::{
    ScriptBuf, XOnlyPublicKey,
    key::Keypair,
    opcodes::all::OP_CHECKSIG,
    script::Builder,
    secp256k1::{SECP256K1, SecretKey},
    taproot::{TaprootBuilder, TaprootSpendInfo},
};

use bitcoind_async_client::Client;

use crate::app::TARGET_BYTE_SIZE;

const CHUNK_SIZE: usize = 64;

pub(crate) struct Wallet {
    rpc_client: Client,
    sk: SecretKey,
    payload: Vec<u8>,
}

impl Wallet {
    pub(crate) fn new(rpc_client: Client, payload: Vec<u8>) -> anyhow::Result<Self> {
        let kp = Keypair::new(SECP256K1, &mut rand::thread_rng());
        let sk = kp.secret_key();
        Ok(Self {
            rpc_client,
            sk,
            payload,
        })
    }

    #[inline]
    fn x_only_public_key(&self) -> XOnlyPublicKey {
        let x_only_public_key = self.sk.x_only_public_key(SECP256K1).0;
        x_only_public_key
    }

    /// Splits the 4096-byte payload into 64-byte chunks.
    /// Returns a Vec of 64-byte arrays ([u8; 64]).
    /// TODO: can this return an iterator?
    fn create_chunks(&self) -> anyhow::Result<Vec<[u8; CHUNK_SIZE]>> {
        if self.payload.len() != TARGET_BYTE_SIZE {
            anyhow::bail!(
                "Payload must be exactly 4096 bytes, got {}",
                self.payload.len()
            );
        }
        let mut chunks = Vec::with_capacity(TARGET_BYTE_SIZE / CHUNK_SIZE);
        for chunk in self.payload.chunks(CHUNK_SIZE) {
            let arr: [u8; CHUNK_SIZE] = chunk.try_into().expect("Chunk size mismatch");
            chunks.push(arr);
        }
        Ok(chunks)
    }

    #[inline]
    fn create_chunk_tapleaf(&self, chunk: [u8; CHUNK_SIZE]) -> anyhow::Result<ScriptBuf> {
        let chunk_script = Builder::new().push_slice(chunk).into_script();
        Ok(chunk_script)
    }

    pub(crate) fn create_taproot_output(&self) -> anyhow::Result<TaprootSpendInfo> {
        let internal_key = self.x_only_public_key();
        let mut builder = TaprootBuilder::new();

        let chunks = self.create_chunks()?;
        let depth = (chunks.len() as u32).ilog2() as u8;
        println!("depth: {}", depth);
        for chunk in chunks {
            let chunk_script = self.create_chunk_tapleaf(chunk)?;
            println!("chunk_script: {:?}", chunk_script.to_asm_string());
            println!("depth: {}", depth);
            builder = builder.add_leaf(depth, chunk_script)?;
        }

        let taproot_info = match builder.finalize(SECP256K1, internal_key) {
            Ok(info) => info,
            Err(builder) => {
                panic!("Failed to create taproot output: {:?}", builder);
            }
        };
        Ok(taproot_info)
    }
}
