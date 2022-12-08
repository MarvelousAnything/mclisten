use std::io;
use color_eyre::eyre::Result;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_buf};
use tokio::{select, try_join};
use crate::packet::Packet;

#[derive(Error, Debug)]
enum ProxyError {
    #[error("Could not connect to server at {addr}")]
    ServerConnection { addr: String },
    #[error("Could not bind proxy to {addr}")]
    ProxyBind { addr: String },
}

pub struct Proxy {
    server_addr: SocketAddr,
    proxy_addr: SocketAddr,
}

impl Proxy {
    pub fn new(
        server_host: String,
        server_port: String,
        proxy_host: String,
        proxy_port: String,
    ) -> Result<Self> {
        let server_addr = format!("{}:{}", server_host, server_port).parse()?;

        let proxy_addr = format!("{}:{}", proxy_host, proxy_port).parse()?;

        Ok(Self {
            server_addr,
            proxy_addr,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting proxy {:?} -> {:?}",
            self.server_addr, self.proxy_addr
        );

        let listener = TcpListener::bind(self.server_addr).await?;

        while let Ok((inbound, _)) = listener.accept().await {
            let transfer = Proxy::transfer(inbound, self.proxy_addr);
            tokio::spawn(transfer);
        }

        Ok(())
    }

    async fn transfer(mut inbound: TcpStream, proxy_addr: SocketAddr) -> Result<()> {
        let mut outbound = TcpStream::connect(proxy_addr).await?;
        let (mut ri, mut wi) = inbound.split();
        let (mut ro, mut wo) = outbound.split();

        let client_to_server = async {
            // log the data we're sending to the server
            let mut buf = [0; 1024];
            loop {
                let n = ri.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                if buf[1] < 100 {
                    info!("Client -> Server");
                    let packet = Packet::from_buffer(&buf[..n]);
                    if let Err(e) = packet {
                        error!("Error parsing packet: {}", e);
                    }
                }
                wo.write_all(&buf[..n]).await?;
            }
            wo.shutdown().await
        };
        let server_to_client = async {
            // log the data we're receiving from the server
            let mut buf = [0; 1024];
            loop {
                let n = ro.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                if buf[1] < 100 {
                    info!("Server -> Client");
                    let packet = Packet::from_buffer(&buf[..n]);
                    if let Err(e) = packet {
                        error!("Error parsing packet: {}", e);
                    }
                }
                wi.write_all(&buf[..n]).await?;
            }
            wi.shutdown().await
        };

        select! {
            _ = client_to_server => {
                info!("Client to server transfer complete");
            }
            _ = server_to_client => {
                info!("Server to client transfer complete");
            }
        }
        Ok(())
    }
}
