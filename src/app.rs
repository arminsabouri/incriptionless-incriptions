use crate::wallet::Wallet;
use bitcoind_async_client::Client;
use hex;
use log::info;
use std::fs;

pub(crate) const TARGET_BYTE_SIZE: usize = 4096;
pub(crate) const TARGET_HEX_LENGTH: usize = TARGET_BYTE_SIZE * 2;

fn create_payload(payload_location: String) -> anyhow::Result<Vec<u8>> {
    let payload_bytes =
        fs::read(payload_location).expect("Failed to read. Make sure the file exists.");

    let mut hex_data = hex::encode(payload_bytes).trim().to_string();

    if hex_data.len() < TARGET_HEX_LENGTH {
        hex_data.extend(std::iter::repeat('0').take(TARGET_HEX_LENGTH - hex_data.len()));
        info!(
            "Hex data was shorter than {} characters, padded with '0's.",
            TARGET_HEX_LENGTH
        );
    } else if hex_data.len() > TARGET_HEX_LENGTH {
        eprintln!(
            "Error: Input file 'labitbu.webp' is too large. Maximum size is {} bytes.",
            TARGET_BYTE_SIZE
        );
        std::process::exit(1);
    }

    Ok(hex::decode(hex_data)?)
}

pub(crate) struct App {
    rpc_client: Client,
    wallet: Wallet,
}

impl App {
    pub(crate) fn try_new(rpc_client: Client, payload_location: String) -> anyhow::Result<Self> {
        let payload_bytes = create_payload(payload_location)?;
        let wallet = Wallet::new(rpc_client.clone(), payload_bytes)?;
        Ok(Self { rpc_client, wallet })
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let taproot_info = self.wallet.create_taproot_output()?;
        Ok(())
    }
}
