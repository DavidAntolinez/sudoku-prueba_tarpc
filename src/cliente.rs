use service::{WorldClient, sudoku::SudokuSize};
use tarpc::{client, context, tokio_serde::formats::Json};
use std::net::SocketAddr;

pub async fn run_client(server_addr: SocketAddr) -> anyhow::Result<()> {
    let mut transport =
        tarpc::serde_transport::tcp::connect(server_addr, Json::default);

    transport.config_mut().max_frame_length(usize::MAX);

    let client =
        WorldClient::new(client::Config::default(), transport.await?).spawn();

    loop {
        println!("\n=== Cliente Sudoku ===");
        println!("1. Hello");
        println!("2. Sudoku 4x4");
        println!("3. Sudoku 9x9");
        println!("4. Sudoku 16x16");
        println!("5. Salir");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => {
                let res = client
                    .hello(context::current(), "Usuario".into())
                    .await?;
                println!("Respuesta: {res}");
            }
            "2" => request_sudoku(&client, SudokuSize::SUDOKU4X4).await?,
            "3" => request_sudoku(&client, SudokuSize::SUDOKU9X9).await?,
            "4" => request_sudoku(&client, SudokuSize::SUDOKU16X16).await?,
            "5" => break,
            _ => println!("Opción inválida"),
        }
    }

    Ok(())
}

async fn request_sudoku(
    client: &WorldClient,
    size: SudokuSize,
) -> Result<(), anyhow::Error> {
    let sudoku = client.sudoku(context::current(), size).await?;

    for row in sudoku.unwrap().board {
        println!("{row:?}");
    }

    Ok(())
}
