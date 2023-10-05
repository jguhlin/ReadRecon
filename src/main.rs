use std::io;
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() {
    use std::{io, thread, time::Duration};
    use ratatui::{
        backend::CrosstermBackend,
        widgets::{Block, Borders},
        Terminal
    };
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    
    fn main() -> Result<(), io::Error> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
    
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("Block")
                .borders(Borders::ALL);
            f.render_widget(block, size);
        })?;
    
        // Start a thread to discard any input events. Without handling events, the
        // stdin buffer will fill up, and be read into the shell when the program exits.
        thread::spawn(|| loop {
            event::read();
        });
    
        thread::sleep(Duration::from_millis(5000));
    
        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
    
        Ok(())
    }
}

// From: https://github.com/wdecoster/chopper/blob/master/src/main.rs#L157
fn ave_qual(quals: &[u8]) -> f64 {
    let probability_sum = quals
        .iter()
        .map(|q| 10_f64.powf((*q as f64) / -10.0))
        .sum::<f64>();
    (probability_sum / quals.len() as f64).log10() * -10.0
}