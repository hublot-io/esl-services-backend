use std::{
    io,
    time::{Duration, Instant},
};


use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame, Terminal,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetState};

pub async fn run_ui<B: Backend>(terminal: &mut Terminal<B>, tick_rate: Duration) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f))?;

        let _timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        // if crossterm::event::poll(timeout)? {
        //     if let Event::Key(key) = event::read()? {
        //         if let KeyCode::Char('q') = key.code {
        //             return Ok(());
        //         }
        //     }
        // }
        if last_tick.elapsed() >= tick_rate {
            // app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.size());

    let title_block = Block::default().title("Hublot PRICER Bridge");
    f.render_widget(title_block, chunks[0]);

    let filter_state = TuiWidgetState::new().set_default_display_level(log::LevelFilter::Debug);
    // .set_level_for_target("New event", log::LevelFilter::Debug)
    // .set_level_for_target("info", log::LevelFilter::Info);
    let tui_w = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title("Independent Tui Logger View")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Magenta))
        .style_info(Style::default().fg(Color::Cyan))
        .output_separator('|')
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Long))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .state(&filter_state);
    f.render_widget(tui_w, chunks[1]);
}
