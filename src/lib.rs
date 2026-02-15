// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};
use self::sudoku::Sudoku;
use self::sudoku::SudokuSize;

/// This is the service definition. It looks a lot like a trait definition.
/// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[tarpc::service]
pub trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
    async fn sudoku(size: SudokuSize) -> Result<Sudoku, String>;
}

/// Initializes an OpenTelemetry tracing subscriber with a OTLP backend.
pub fn init_tracing(
    service_name: &'static str,
) -> anyhow::Result<opentelemetry_sdk::trace::SdkTracerProvider> {
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_service_name(service_name)
                .build(),
        )
        .with_batch_exporter(
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .build()
                .unwrap(),
        )
        .build();
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    let tracer = tracer_provider.tracer(service_name);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    Ok(tracer_provider)
}

pub mod sudoku {
    use rand::seq::SliceRandom;
    use rand::{rng};
    use tarpc::serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Sudoku {
        pub board: Vec<Vec<u8>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum SudokuSize {
        SUDOKU4X4,
        SUDOKU9X9,
        SUDOKU16X16,
    }

    impl Sudoku {
        pub async fn generate_sudoku(size: SudokuSize) -> Result<Sudoku, String> {
            let box_size = match size {
                SudokuSize::SUDOKU4X4 => 2,
                SudokuSize::SUDOKU9X9 => 3,
                SudokuSize::SUDOKU16X16 => 4,
            };

            let n = box_size * box_size;

            let mut board = vec![vec![0u8; n]; n];

            if !fill_board(&mut board, box_size) {
                return Err("No se pudo generar el sudoku".into());
            }

            // quitar celdas para hacer puzzle
            let empty_cells = match size {
                SudokuSize::SUDOKU4X4 => 6,
                SudokuSize::SUDOKU9X9 => 40,
                SudokuSize::SUDOKU16X16 => 120,
            };

            remove_cells(&mut board, empty_cells);

            Ok(Sudoku { board })
        }
    }

    fn fill_board(board: &mut Vec<Vec<u8>>, box_size: usize) -> bool {
        let n = board.len();

        for row in 0..n {
            for col in 0..n {
                if board[row][col] == 0 {
                    let mut nums: Vec<u8> = (1..=n as u8).collect();
                    nums.shuffle(&mut rng());

                    for num in nums {
                        if is_valid(board, row, col, num, box_size) {
                            board[row][col] = num;
                            if fill_board(board, box_size) {
                                return true;
                            }
                            board[row][col] = 0;
                        }
                    }

                    return false;
                }
            }
        }

        true
    }

    fn is_valid(
        board: &Vec<Vec<u8>>,
        row: usize,
        col: usize,
        num: u8,
        box_size: usize,
    ) -> bool
    {
        let n = board.len();

        // fila
        if board[row].contains(&num) {
            return false;
        }

        // columna
        for r in 0..n {
            if board[r][col] == num {
                return false;
            }
        }

        // caja
        let start_row = (row / box_size) * box_size;
        let start_col = (col / box_size) * box_size;

        for r in 0..box_size {
            for c in 0..box_size {
                if board[start_row + r][start_col + c] == num {
                    return false;
                }
            }
        }

        true
    }

    fn remove_cells(board: &mut Vec<Vec<u8>>, empty: usize) {
        let n = board.len();

        let mut rng = rng();

        let mut cells: Vec<(usize, usize)> = (0..n)
            .flat_map(|r| (0..n).map(move |c| (r, c)))
            .collect();

        cells.shuffle(&mut rng);

        for &(r, c) in cells.iter().take(empty) {
            board[r][c] = 0;
        }
    }
}
