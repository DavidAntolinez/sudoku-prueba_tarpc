mod server;
mod client;

use std::io::{self, stdout};
use std::sync::{Arc, Mutex};
use crossterm::{event::{self, Event, KeyCode}, terminal::{enable_raw_mode, disable_raw_mode}, ExecutableCommand};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use ratatui::widgets::Wrap;
use service::{init_tracing, LogBuffers};
use service::sudoku::Sudoku;
use crate::client::RPCClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    unsafe {std::env::set_var("RUST_LOG", "info,cliente=debug,server=debug,rpc=trace")}

    let buffers = LogBuffers {
        client: Arc::new(Mutex::new(Vec::new())),
        server: Arc::new(Mutex::new(Vec::new())),
        rpc: Arc::new(Mutex::new(Vec::new())),
    };

    tokio::spawn(async move {
        let _ = server::run_server(2001).await;
    });

    init_tracing("sudoku app", buffers.clone())?;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let client = RPCClient::new("[::1]:2001".parse().unwrap()).await;
    let mut app = App {
        client,
        sudoku: None,
        scroll_cliente: 0,
        scroll_server: 0,
        scroll_rpc: 0,
    };

    loop {
        terminal.draw(|f| draw_ui(f, &buffers, &mut app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => {
                        let sudoku = app.client.sudoku4x4().await;
                        app.sudoku = Some(sudoku)
                    },
                    KeyCode::Char('2') => {
                        let sudoku = app.client.sudoku9x9().await;
                        app.sudoku = Some(sudoku)
                    },
                    KeyCode::Char('3') => {
                        let sudoku = app.client.sudoku16x16().await;
                        app.sudoku = Some(sudoku)
                    },
                    KeyCode::Char('q') | KeyCode::Char('4') => {break;},
                    _ => {}
                }
            }
        }

    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    // ratatui::restore();

    Ok(())
}

struct App {
    client: RPCClient,
    sudoku: Option<Sudoku>,
    scroll_cliente: u16,
    scroll_server: u16,
    scroll_rpc: u16,
}


fn draw_ui(frame: &mut Frame, buffers: &LogBuffers, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(50),
            Constraint::Min(50),
        ])
        .split(frame.size());

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),   // menú
            Constraint::Min(10),     // sudoku
        ])
        .split(layout[0]);
2;
    let menu = Paragraph::new("1. Sudoku 4x4 \n2. Sudoku 9x9 \n3. Sudoku 16x16 \n4 o q. Salir")
        .wrap( Wrap::default() )
        .block(Block::default().title("Menu").borders(Borders::ALL));

    frame.render_widget(menu, left[0]);

    let sudoku = sudoku_widget(app.sudoku.as_ref());
    frame.render_widget(sudoku, left[1]);


    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(10),
            Constraint::Percentage(50),
        ])
        .split(layout[1]);

    let cliente_logs = buffers.client.lock().unwrap().join("");
    let cliente_lines = cliente_logs.lines().count() as u16;
    app.scroll_cliente =
        cliente_lines.saturating_sub(right[0].height);

    let server_logs = buffers.server.lock().unwrap().join("");
    let server_lines = server_logs.lines().count() as u16;
    app.scroll_server =
        server_lines.saturating_sub(right[1].height);

    let rpc_logs = buffers.rpc.lock().unwrap().join("");
    let rpc_lines = rpc_logs.lines().count() as u16;
    app.scroll_rpc =
        rpc_lines.saturating_sub(right[2].height);

    let cliente = Paragraph::new(cliente_logs)
        .wrap(Wrap::default())
        .scroll((app.scroll_cliente, 0))
        .block(Block::default().title("Cliente").borders(Borders::ALL));

    let server = Paragraph::new(server_logs)
        .wrap(Wrap::default())
        .scroll((app.scroll_server, 0))
        .block(Block::default().title("Server").borders(Borders::ALL));

    let rpc = Paragraph::new(rpc_logs)
        .wrap(Wrap::default())
        .scroll((app.scroll_rpc, 0))
        .block(Block::default().title("RPC").borders(Borders::ALL));


    frame.render_widget(cliente, right[0]);
    frame.render_widget(server, right[1]);
    frame.render_widget(rpc, right[2]);
}






pub fn sudoku_widget(sudoku: Option<&Sudoku>) -> Paragraph<'static> {
    let text = if let Some(s) = sudoku {
        render_board(&s.board)
    } else {
        "No hay sudoku".to_string()
    };

    Paragraph::new(text)
        .block(Block::default().title("Sudoku").borders(Borders::ALL))
        .alignment(Alignment::Center)
}

fn render_board(board: &[Vec<u8>]) -> String {
    let mut out = String::new();

    for row in board {
        for &cell in row {
            if cell == 0 {
                out.push_str(" . ");
            } else {
                out.push_str(&format!("{:^3}", cell));
            }
        }
        out.push('\n');
    }

    out
}



