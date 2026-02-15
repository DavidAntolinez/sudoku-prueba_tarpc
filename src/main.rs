mod server;
mod cliente;

use std::net::{IpAddr, Ipv6Addr, SocketAddr};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = 50051;

    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port);

    // lanzar servidor en background
    tokio::spawn(async move {
        if let Err(e) = server::run_server(port).await {
            eprintln!("Server error: {e}");
        }
    });

    // pequeña espera para que el server arranque
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // lanzar cliente interfaz
    cliente::run_client(addr).await?;

    Ok(())
}
