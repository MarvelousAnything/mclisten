#![feature(cursor_remaining)]
use clap::Parser;
use color_eyre::eyre::Result;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use proxy::Proxy;

mod proxy;
mod util;
mod packet;

#[derive(Parser)]
#[command(name = "MC Listen")]
#[command(author = "Owen V. Hayes <owen.v.hayes@gmail.com>")]
#[command(version = "0.1.0")]
#[command(about = "A Minecraft Packet Listener")]
struct Cli {
    #[arg(long)]
    server_port: Option<String>,
    #[arg(long)]
    proxy_port: Option<String>,
    #[arg(long)]
    server_host: Option<String>,
    #[arg(long)]
    proxy_host: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();
    let cli = Cli::parse();

    let proxy = Proxy::new(
        cli.server_host.unwrap_or_else(|| "127.0.0.1".to_string()),
        cli.server_port.unwrap_or_else(|| "25565".to_string()),
        cli.proxy_host.unwrap_or_else(|| "0.0.0.0".to_string()),
        cli.proxy_port.unwrap_or_else(|| "25565".to_string()),
    )?;
    proxy.start().await?;
    Ok(())
}
