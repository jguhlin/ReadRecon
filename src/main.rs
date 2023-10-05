use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() {
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend,
        widgets::{Block, Borders},
        Terminal,
    };
    use std::{io, thread, time::Duration};

}

// From: https://github.com/wdecoster/chopper/blob/master/src/main.rs#L157
fn ave_qual(quals: &[u8]) -> f64 {
    let probability_sum = quals
        .iter()
        .map(|q| 10_f64.powf((*q as f64) / -10.0))
        .sum::<f64>();
    (probability_sum / quals.len() as f64).log10() * -10.0
}
