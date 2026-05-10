use std::io::{self, Write};
use std::sync::OnceLock;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bitcoin::hashes::{sha256, Hash};

const NUM_LINES: usize = 9;
const MIDDLE: usize = (NUM_LINES - 1) / 2;
const INTERVAL: Duration = Duration::from_millis(100);
const MIN_DISPLAY: Duration = Duration::from_secs(1);
static CTRL_C_HANDLER: OnceLock<()> = OnceLock::new();

fn sha256_hex(input: &[u8]) -> String {
    hex::encode(sha256::Hash::hash(input).to_byte_array())
}

fn render_overlay(out: &mut impl Write, hash: &str, text: &str) {
    let trimmed = text.trim_end_matches("...").to_lowercase();
    let display = if trimmed.len() == hash.len() {
        trimmed
    } else {
        format!(" {trimmed} ")
    };
    let h_len = hash.len();
    let t_len = display.len();
    if t_len >= h_len {
        writeln!(out, "\x1b[0m{}", &display[..h_len]).ok();
    } else {
        let start = (h_len - t_len) / 2;
        let end = start + t_len;
        writeln!(
            out,
            "\x1b[90m{}\x1b[0m{}\x1b[0;90m{}\x1b[0m",
            &hash[..start],
            display,
            &hash[end..],
        )
        .ok();
    }
}

fn render(out: &mut impl Write, lines: &[String], status: &str) {
    let status_lines: Vec<&str> = status.lines().collect();
    write!(out, "\x1b[{NUM_LINES}A").ok();
    for (i, hash) in lines.iter().enumerate() {
        write!(out, "\r\x1b[2K").ok();
        if i >= MIDDLE {
            if let Some(text) = status_lines.get(i - MIDDLE) {
                render_overlay(out, hash, text);
            } else {
                writeln!(out, "\x1b[90m{hash}\x1b[0m").ok();
            }
        } else {
            writeln!(out, "\x1b[90m{hash}\x1b[0m").ok();
        }
    }
    write!(out, "\x1b[J").ok();
    out.flush().ok();
}

struct State {
    status: String,
    write_gen: u64,
    render_gen: u64,
    rendered_at: Option<Instant>,
    running: bool,
}

pub struct Animation {
    shared: Arc<(Mutex<State>, Condvar)>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Animation {
    pub fn start(seed: &str) -> Self {
        CTRL_C_HANDLER.get_or_init(|| {
            ctrlc::set_handler(|| {
                let mut out = io::stderr();
                write!(out, "\x1b[?25h").ok();
                out.flush().ok();
                std::process::exit(130);
            })
            .expect("failed to install Ctrl-C handler");
        });

        // Build initial lines: each is sha256 of the previous
        let mut lines = Vec::with_capacity(NUM_LINES);
        let mut prev = sha256_hex(seed.as_bytes());
        lines.push(prev.clone());
        for _ in 1..NUM_LINES {
            prev = sha256_hex(prev.as_bytes());
            lines.push(prev.clone());
        }

        let shared = Arc::new((
            Mutex::new(State {
                status: String::new(),
                write_gen: 0,
                render_gen: 0,
                rendered_at: None,
                running: true,
            }),
            Condvar::new(),
        ));
        let shared_c = Arc::clone(&shared);

        // Claim vertical space for the animation block and hide the cursor.
        {
            let mut out = io::stderr();
            write!(out, "\x1b[?25l").ok();
            for _ in 0..NUM_LINES {
                writeln!(out).ok();
            }
            out.flush().ok();
        }

        let handle = thread::spawn(move || {
            let mut out = io::stderr();
            let mut next_tick = Instant::now() + INTERVAL;

            loop {
                let s = shared_c.0.lock().unwrap();
                if !s.running {
                    break;
                }
                let text = s.status.clone();
                let gen = s.write_gen;
                drop(s);

                render(&mut out, &lines, &text);

                // Mark rendered and notify any blocking set_status caller
                let mut s = shared_c.0.lock().unwrap();
                s.render_gen = gen;
                s.rendered_at = Some(Instant::now());
                shared_c.1.notify_all();

                // Wait until next scheduled tick or wake from set_status/drop
                let timeout = next_tick.saturating_duration_since(Instant::now());
                let (guard, _) = shared_c.1.wait_timeout(s, timeout).unwrap();
                drop(guard);

                // Advance hashes on the regular cadence
                if Instant::now() >= next_tick {
                    let new = sha256_hex(lines.last().unwrap().as_bytes());
                    lines.remove(0);
                    lines.push(new);
                    next_tick += INTERVAL;
                }
            }
        });

        Animation {
            shared,
            handle: Some(handle),
        }
    }

    /// Update the status text and block until it has been rendered.
    /// Ensures the previous status was visible for at least MIN_DISPLAY.
    pub fn set_status(&self, text: &str) {
        // Wait for previous status to have been shown long enough
        let remaining = self.shared.0.lock().unwrap().rendered_at
            .and_then(|t| MIN_DISPLAY.checked_sub(t.elapsed()));
        if let Some(d) = remaining {
            thread::sleep(d);
        }

        let mut s = self.shared.0.lock().unwrap();
        s.status = text.to_string();
        s.write_gen += 1;
        let target = s.write_gen;
        self.shared.1.notify_all();
        while s.render_gen < target && s.running {
            s = self.shared.1.wait(s).unwrap();
        }
    }
}

impl Drop for Animation {
    fn drop(&mut self) {
        {
            let mut s = self.shared.0.lock().unwrap();
            s.running = false;
            self.shared.1.notify_all();
        }
        if let Some(h) = self.handle.take() {
            h.join().ok();
        }
        let mut out = io::stderr();
        write!(out, "\x1b[?25h").ok();
        out.flush().ok();
    }
}
