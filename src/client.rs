use service::{WorldClient, sudoku::SudokuSize};
use tarpc::{client, context, tokio_serde::formats::Json};
use std::net::SocketAddr;
use service::sudoku::{Sudoku};

pub struct RPCClient {
    rpc: WorldClient
}

impl RPCClient {
    pub async fn new(addr: SocketAddr) -> Self {
        let mut transport = tarpc::serde_transport::tcp::connect(addr, Json::default);
        transport.config_mut().max_frame_length(usize::MAX);
        let client = WorldClient::new(client::Config::default(), transport.await.unwrap()).spawn();
        tracing::info!(target: "cliente", "Cliente inicializado");
        Self {
            rpc: client
        }
    }

    pub async fn sudoku4x4(&self) -> Sudoku {
        self.request_sudoku(SudokuSize::SUDOKU4X4).await.unwrap()
    }

    pub async fn sudoku9x9(&self) -> Sudoku {
        self.request_sudoku(SudokuSize::SUDOKU9X9).await.unwrap()
    }

    pub async fn sudoku16x16(&self) -> Sudoku {
        self.request_sudoku(SudokuSize::SUDOKU16X16).await.unwrap()
    }

    async fn request_sudoku(&self, size: SudokuSize, ) -> Result<Sudoku, anyhow::Error> {
        let sudoku = self.rpc.sudoku(context::current(), size).await?.unwrap();
        let mut buffer = String::new();
        for row in &sudoku.board {
            buffer.push_str(&format!("{row:?}\n"));
        }
        tracing::info!(target: "cliente", "SUDOKU: {}", buffer);
        Ok(sudoku)
    }

    pub async fn check_sudoku(&self, sudoku: &mut Sudoku) {
        let state = self.rpc.is_solved(context::current(), sudoku.clone()).await;
        sudoku.state = state.expect("REASON")
    }
}