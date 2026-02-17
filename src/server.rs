use futures::{future, prelude::*};
use rand::{
    distr::{Distribution, Uniform},
    rng,
};
use service::{World};
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    time::Duration,
};
use tarpc::{
    context,
    server::{self, Channel, incoming::Incoming},
    tokio_serde::formats::Json,
};
use service::sudoku::{Sudoku, SudokuSize, SudokuState};
use tokio::time;

#[derive(Clone)]
struct HelloServer(SocketAddr);

impl World for HelloServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        let sleep_time =
            Duration::from_millis(Uniform::new_inclusive(1, 10).unwrap().sample(&mut rng()));
        time::sleep(sleep_time).await;
        format!("Hello, {name}! You are connected from {}", self.0)
    }

    async fn sudoku(self, _: context::Context, size: SudokuSize) -> Result<Sudoku, String> {
        Sudoku::generate_sudoku(size).await
    }

    async fn is_solved(self, _: context::Context, sudoku: Sudoku) -> SudokuState {
        sudoku.clone().check_user_board(&sudoku.board, sudoku.sudoku_size)
    }
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), port);
    let mut listener =
        tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;

    listener.config_mut().max_frame_length(usize::MAX);

    println!("Servidor escuchando en {}", listener.local_addr());
    tracing::info!(target: "server", "Server Up");

    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        .map(|channel| {
            let server = HelloServer(channel.transport().peer_addr().unwrap());
            channel.execute(server.serve()).for_each(spawn)
        })
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}

