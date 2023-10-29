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

use flate2::read::GzDecoder;
use humansize::{format_size, DECIMAL};
use needletail::*;
use ratatui::{prelude::*, widgets::*};

#[derive(Debug, Clone)]
struct ReadStats {
    qualities: Vec<u8>,
    lengths: Vec<u32>,
    duplex: (u64, u64, u64),
}

impl ReadStats {
    fn quals_as_data(&self) -> Vec<(f64, f64)> {
        let mut data = Vec::new();
        for (i, q) in self.qualities.iter().enumerate() {
            data.push((i as f64, *q as f64));
        }
        data
    }

    fn lengths_as_data(&self) -> Vec<(f64, f64)> {
        let mut data = Vec::new();
        for (i, q) in self.lengths.iter().enumerate() {
            data.push((i as f64, *q as f64));
        }
        data
    }
}

// Given a set of data (Vec<u64 or u8>), return a histogram, with nbins as an argument
fn histogram<T: num::Unsigned>(data: &[T], nbins: usize) -> Vec<(String, T)> 
where
    &T: std::ops::Add<usize>,
{
    let mut hist = Vec::new();
    let max = data.iter().max().unwrap();
    let min = data.iter().min().unwrap();
    let bin_size = (max - min) / nbins as usize;

    for i in 0..nbins {
        let lower = min + (i * bin_size);
        let upper = min + ((i + 1) * bin_size);
        let label = format!("{}-{}", lower, upper);
        let count = data.iter().filter(|&x| *x >= lower && *x < upper).count();
        hist.push((label, count as T));
    }

    hist
}

struct App {
    stats: ReadStats,
}

impl App {
    fn new() -> App {
        App {
            stats: ReadStats {
                qualities: Vec::new(),
                lengths: Vec::new(),
                duplex: (0, 0, 0),
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
    app.stats.qualities = vec![10, 25, 10, 254, 200, 10, 1];
    app.stats.lengths = vec![10, 25, 10, 254, 205550, 10, 1];
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
    let stdin = io::stdin();
    let mut filetype = None;

    let mut to_process = Vec::new();
    let mut buf = [0; 1024];
    let mut lines = Vec::new();

    loop {
        // Read from stdin
        let mut stdin = stdin.lock();
        let bytes_read = stdin.read(&mut buf)?;
        drop(stdin);
        // Append to to_process
        to_process.extend_from_slice(&buf[..bytes_read]);

        if bytes_read > 0 {
            if filetype.is_none() && to_process.len() >= 1024 {
                filetype = Some(detect_filetype(&buf[..1024]).unwrap());
            }
        }

        if to_process.len() > 1024 && filetype.is_some() {
            // Process the buffer
            match filetype.unwrap() {
                FileType::Fastq => {
                    // Just process lines
                    lines = to_process.split(|&x| x == b'\n').collect();

                    // Process 4 at a time, pausing if there aren't at least 4
                    while lines.len() >= 4 {
                        let seq_line = lines.remove(1);
                        let seq_line = String::from_utf8(seq_line.to_vec()).unwrap();
                        let seq_len = seq_line.trim().len();
                        app.stats.lengths.push(seq_len as u32);

                        let qual_line = lines.remove(3);
                        let qual_line = String::from_utf8(qual_line.to_vec()).unwrap();
                        let quals = qual_line.trim().as_bytes();
                        let ave_qual = ave_qual(quals);
                        app.stats.qualities.push(ave_qual as u8);

                        lines = lines.split_off(4);
                    }
                }
                _ => {
                    println!("Unsupported file type or not progrrammed in yet");
                }
            }
        }

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

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
        .split(size);

    let quals_hist = histogram(&app.stats.qualities, 10);
    let mut barchart = BarChart::default()
        .block(
            Block::default()
                .title("Quality Scores")
                .borders(Borders::ALL),
        )
        .bar_width(8)
        .group_gap(5);

    let group = {
        let bars: Vec<Bar> = quals_hist
            .into_iter()
            // .text_value(label.clone())
            .map(|(label, value)| {
                Bar::default()
                    .value(value as u64)
                    .text_value("".to_string())
                    .label(label.into())
            })
            .collect();
        BarGroup::default().bars(&bars)
    };

    barchart = barchart.data(group);

    f.render_widget(barchart, chunks[0]);

    let lengths_hist = histogram(&app.stats.lengths, 10);
    let mut barchart = BarChart::default()
        .block(Block::default().title("Read Lengths").borders(Borders::ALL))
        .bar_width(20)
        .group_gap(20);

    let group = {
        let bars: Vec<Bar> = lengths_hist
            .into_iter()
            .map(|(label, value)| {
                Bar::default()
                    .value(value as u64)
                    .text_value("".to_string())
                    .label(label.into())
            })
            .collect();
        BarGroup::default().bars(&bars)
    };

    barchart = barchart.data(group);

    f.render_widget(barchart, chunks[1]);
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
