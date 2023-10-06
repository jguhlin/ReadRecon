use std::{
    error::Error,
    io,
    io::Read,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fffx::*;
use flate2::read::GzDecoder;
use ratatui::{prelude::*, widgets::*};

#[derive(Debug, Clone)]
struct ReadStats {
    read_qualities: Vec<u8>,
    read_lengths: Vec<u32>,
}

impl ReadStats {
    fn quals_as_data(&self) -> Vec<(f64, f64)> {
        let mut data = Vec::new();
        for (i, q) in self.read_qualities.iter().enumerate() {
            data.push((i as f64, *q as f64));
        }
        data
    }

    fn lengths_as_data(&self) -> Vec<(f64, f64)> {
        let mut data = Vec::new();
        for (i, q) in self.read_lengths.iter().enumerate() {
            data.push((i as f64, *q as f64));
        }
        data
    }

    fn quals_histogram(&self) -> Vec<(f64, f64)> {
        let mut data = Vec::new();

        // Calculate a reasonable number of bins based on min/max values
        let min = self.read_qualities.iter().min().unwrap();
        let max = self.read_qualities.iter().max().unwrap();
        let bin_size = (max - min) / 10;
        
        


        
    }

}

struct App {
    stats: ReadStats,
}

impl App {
    fn new() -> App {
        App {
            stats: ReadStats {
                read_qualities: Vec::new(),
                read_lengths: Vec::new(),
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();

    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let mut app = App::new();
    app.stats.read_qualities = vec![10, 25, 10, 254, 200, 10, 1];
    app.stats.read_lengths = vec![10, 25, 10, 254, 200, 10, 1];
    let res = run_app(&mut terminal, app, tick_rate);

    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ]
            .as_ref(),
        )
        .split(size);

    let x_labels = vec![
        Span::styled("Hello", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("Hello2", Style::default().add_modifier(Modifier::BOLD)),
    ];

    let quals = app
        .stats
        .read_qualities
        .iter()
        .enumerate()
        .map(|(i, q)| (i as f64, *q as f64))
        .collect::<Vec<(f64, f64)>>();
    let lengths = app
        .stats
        .read_qualities
        .iter()
        .enumerate()
        .map(|(i, q)| (i as f64, *q as f64))
        .collect::<Vec<(f64, f64)>>();

    let datasets = vec![Dataset::default()
        .name("Quals")
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Cyan))
        .data(&quals)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("Chart 1".cyan().bold())
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("X Axis")
                .style(Style::default().fg(Color::Gray))
                .labels(x_labels)
                .bounds([0.0, 10.0]),
        )
        .y_axis(
            Axis::default()
                .title("Y Axis")
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["-20".bold(), "0".into(), "20".bold()])
                .bounds([-20.0, 20.0]),
        );
    f.render_widget(chart, chunks[0]);

    // Lengths dataset
    let x_labels = vec![
        Span::styled("Hello", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("Hello2", Style::default().add_modifier(Modifier::BOLD)),
    ];

    let datasets = vec![Dataset::default()
        .name("data3")
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Yellow))
        .data(&lengths)];

        let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("Chart 2".cyan().bold())
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("X Axis")
                .style(Style::default().fg(Color::Gray))
                .labels(x_labels)
                .bounds([0.0, 10.0]),
        )
        .y_axis(
            Axis::default()
                .title("Y Axis")
                .style(Style::default().fg(Color::Gray))
                .labels(vec!["-20".bold(), "0".into(), "20".bold()])
                .bounds([-20.0, 20.0]),
        );
    f.render_widget(chart, chunks[1]);
}

// From: https://github.com/wdecoster/chopper/blob/master/src/main.rs#L157
fn ave_qual(quals: &[u8]) -> f64 {
    let probability_sum = quals
        .iter()
        .map(|q| 10_f64.powf((*q as f64) / -10.0))
        .sum::<f64>();
    (probability_sum / quals.len() as f64).log10() * -10.0
}

/// Possible file types enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Fasta,
    Fastq,
    Sam,
    Bam,
    Cram,
}

/// Error Type for Detect Filetype
#[derive(Debug, Clone)]
pub enum FileFormatDetectionError {
    UnknownFileType,
}

/// Detect file type
/// Supported file types: FASTA, FASTQ, SAM, BAM, CRAM
/// Returns file type as an enum
/// Works directly from buffer (so no data is consumed)
fn detect_filetype(buf: &[u8]) -> Result<FileType, FileFormatDetectionError> {
    // FASTA files are plaintext and start with ">"
    if buf[0] == b'>' {
        return Ok(FileType::Fasta);
    // SAM files start with @HD     VN:
    } else if buf[0] == b'@' && buf[1] == b'H' && buf[2] == b'D' {
        return Ok(FileType::Sam);
    // FASTQ files are plaintext and start with "@"
    } else if buf[0] == b'@' {
        return Ok(FileType::Fastq);
    // CRAM files start with CRAM
    } else if buf[0] == b'C' && buf[1] == b'R' && buf[2] == b'A' && buf[3] == b'M' {
        return Ok(FileType::Cram);
    }

    // BAM files are compressed, but then start with BAM\1
    // So need at least 100 bytes to check
    if buf.len() >= 100 {
        let mut gz = GzDecoder::new(&buf[..]);
        let mut buf = [0; 3];
        if gz.read_exact(&mut buf).is_ok() && &buf == b"BAM" {
            return Ok(FileType::Bam);
        }
    }

    Err(FileFormatDetectionError::UnknownFileType)
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_filetype() {
        let fasta = b">test\nACGT";
        let fastq = b"@test\nACGT\n+\nIIII";
        let sam = b"@HD\tVN:1.6";
        let cram = b"CRAM";

        // TODO: Add bam test

        assert_eq!(detect_filetype(fasta).unwrap(), FileType::Fasta);
        assert_eq!(detect_filetype(fastq).unwrap(), FileType::Fastq);
        assert_eq!(detect_filetype(sam).unwrap(), FileType::Sam);
        assert_eq!(detect_filetype(cram).unwrap(), FileType::Cram);
    }
}
