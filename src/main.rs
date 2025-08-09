use anyhow::Result;
use bitcoind_async_client::Client;
use clap::Parser;

mod app;
mod wallet;
// Command line arguments
#[derive(Clone, Debug, Parser)]
struct Args {
    // TODO support cookie file, async library first needs to support this
    #[clap(long)]
    bitcoind_user: String,
    #[clap(long)]
    bitcoind_password: String,
    #[clap(long)]
    bitcoind_host: String,
    #[clap(long)]
    bitcoind_rpc_port: u16,
    #[clap(long)]
    image_location: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    log::info!("welcome to mempool tracker");
    env_logger::init();

    let args = Args::parse();
    let bitcoind_url = format!("http://{}:{}", args.bitcoind_host, args.bitcoind_rpc_port);

    let rpc_client = Client::new(
        bitcoind_url,
        args.bitcoind_user,
        args.bitcoind_password,
        None,
        None,
    )?;

    let app = app::App::try_new(rpc_client, args.image_location)?;
    app.run()?;

    Ok(())
}
