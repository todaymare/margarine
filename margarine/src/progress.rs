use std::cell::RefCell;
use std::io::{self, IsTerminal, Read};
use std::time::{Duration, Instant};

use colourful::ColourBrush;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

const TICK_INTERVAL: Duration = Duration::from_millis(80);
const PROGRESS_BAR_DELAY: Duration = Duration::from_millis(100);
const BAR_CHARS: &str = "█▓░";
fn timing_enabled() -> bool {
    std::env::var_os("MARGARINE_TIMING").is_some()
}


fn style(template: &'static str) -> ProgressStyle {
    ProgressStyle::with_template(template)
        .expect("static progress template is valid")
}


fn spinner_style() -> ProgressStyle {
    style("{spinner:.green} {msg}")
}




fn bar_style(template: &'static str) -> ProgressStyle {
    style(template).progress_chars(BAR_CHARS)
}


fn stderr_draw_target(visible: bool) -> ProgressDrawTarget {
    if visible && io::stderr().is_terminal() {
        ProgressDrawTarget::stderr_with_hz(12)
    } else {
        ProgressDrawTarget::hidden()
    }
}


pub fn item_progress(total: u64, message: impl Into<String>) -> AdaptiveProgress {
    let draw_target =
    if io::stdout().is_terminal() {
        ProgressDrawTarget::stdout_with_hz(12)
    } else {
        ProgressDrawTarget::hidden()
    };
    let progress = AdaptiveProgress::new(
        Some(total),
        draw_target,
        Some(bar_style(
            "{spinner:.green} {msg} {bar:40.cyan/blue} {pos}/{len} ({eta})",
        )),
    );
    progress.set_message(message);
    progress
}


struct ActiveCompilation {
    bar: ProgressBar,
    stage_started: Instant,
    timing: bool,
}

thread_local! {
    static ACTIVE_COMPILE_BAR: RefCell<Option<ActiveCompilation>> = const { RefCell::new(None) };
}
impl ActiveCompilation {
    fn finish_stage(&self) {
        if !self.timing {
            return;
        }

        let duration = self.stage_started.elapsed();
        self.bar.println(format!(
            "{} {} ({duration:?})",
            "✓".green().bold(),
            self.bar.message(),
        ));
    }
}


#[derive(Debug, Eq, PartialEq)]
enum StatusKind {
    CompileOwner,
    Independent,
    Borrowed {
        previous: String,
        stage_started: Instant,
    },
}


/// A transient status line. Compilation owns one line per thread; nested
/// compilation and linker statuses temporarily reuse it and restore its
/// previous message when they finish.
pub struct StatusLine {
    bar: ProgressBar,
    kind: StatusKind,
    finished: Option<String>,
}


impl StatusLine {
    pub fn start(message: impl Into<String>) -> Self {
        Self::begin(message.into(), false, true, false)
    }

    pub fn start_compilation(message: impl Into<String>, visible: bool) -> Self {
        Self::begin(message.into(), true, visible, timing_enabled())
    }

    fn begin(message: String, compilation: bool, visible: bool, timing: bool) -> Self {
        ACTIVE_COMPILE_BAR.with(|active| {
            let mut active = active.borrow_mut();
            if let Some(active_compilation) = active.as_mut() {
                let previous = active_compilation.bar.message().to_string();
                let stage_started = active_compilation.stage_started;
                if !compilation {
                    active_compilation.finish_stage();
                    active_compilation.bar.set_message(message);
                    active_compilation.stage_started = Instant::now();
                }
                return Self {
                    bar: active_compilation.bar.clone(),
                    kind: StatusKind::Borrowed { previous, stage_started },
                    finished: None,
                };
            }

            let bar = ProgressBar::with_draw_target(None, stderr_draw_target(visible));
            bar.enable_steady_tick(TICK_INTERVAL);
            bar.set_style(spinner_style());
            bar.set_message(message);
            let kind =
            if compilation {
                *active = Some(ActiveCompilation {
                    bar: bar.clone(),
                    stage_started: Instant::now(),
                    timing,
                });
                StatusKind::CompileOwner
            } else {
                StatusKind::Independent
            };
            Self { bar, kind, finished: None }
        })
    }

    pub(crate) fn set_compilation_message(message: impl Into<String>) {
        let message = message.into();
        ACTIVE_COMPILE_BAR.with(|active| {
            let mut active_compilation = active.borrow_mut();
            let Some(active_compilation) = active_compilation.as_mut()
            else { return };
            if active_compilation.bar.message() == message {
                return;
            }
            active_compilation.finish_stage();
            active_compilation.bar.set_message(message);
            active_compilation.stage_started = Instant::now();
        });
    }


    pub fn suspend<T>(&self, operation: impl FnOnce() -> T) -> T {
        self.bar.suspend(operation)
    }


    pub fn finish(mut self, message: impl Into<String>) {
        self.finished = Some(message.into());
    }


    pub fn clear(self) {}
}


impl Drop for StatusLine {
    fn drop(&mut self) {
        if self.kind == StatusKind::CompileOwner {
            ACTIVE_COMPILE_BAR.with(|active| {
                let active_compilation = active.borrow_mut().take();
                if let Some(active_compilation) = active_compilation {
                    active_compilation.finish_stage();
                }
            });
        }
        match &self.kind {
            StatusKind::CompileOwner | StatusKind::Independent => {
                if let Some(message) = self.finished.take() {
                    self.bar.set_style(style("{msg}"));
                    self.bar.finish_with_message(
                        format!("{} {}", "✓".green().bold(), message),
                    );
                } else {
                    self.bar.finish_and_clear();
                }
            },
            StatusKind::Borrowed { previous, stage_started } => {
                ACTIVE_COMPILE_BAR.with(|active| {
                    let mut active_compilation = active.borrow_mut();
                    let Some(active_compilation) = active_compilation.as_mut()
                    else { return };
                    active_compilation.finish_stage();
                    active_compilation.bar.set_message(previous.clone());
                    active_compilation.stage_started = *stage_started;
                });
            },
        }
    }
}




pub struct AdaptiveProgress {
    bar: ProgressBar,
    expanded_style: RefCell<Option<ProgressStyle>>,
    started: Instant,
}


impl AdaptiveProgress {
    fn new(
        total: Option<u64>,
        draw_target: ProgressDrawTarget,
        expanded_style: Option<ProgressStyle>,
    ) -> Self {
        let bar = ProgressBar::with_draw_target(total, draw_target);
        bar.set_style(spinner_style());
        Self {
            bar,
            expanded_style: RefCell::new(expanded_style),
            started: Instant::now(),
        }
    }

    fn expand_if_ready(&self) {
        if self.started.elapsed() < PROGRESS_BAR_DELAY {
            return;
        }
        let Some(style) = self.expanded_style.borrow_mut().take() else {
            return;
        };
        self.bar.set_style(style);
    }


    pub fn set_message(&self, message: impl Into<String>) {
        self.bar.set_message(message.into());
    }

    pub fn inc(&self, amount: u64) {
        if amount == 0 {
            return;
        }
        self.expand_if_ready();
        self.bar.inc(amount);
    }

    pub fn suspend<T>(&self, operation: impl FnOnce() -> T) -> T {
        self.bar.suspend(operation)
    }

    pub fn finish(self) {
        self.bar.finish_and_clear();
    }
}


pub fn byte_progress(total: Option<u64>) -> AdaptiveProgress {
    let expanded_style =
    total.map(|_| {
        bar_style(
            "{spinner:.green} {msg} {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})",
        )
    });
    let progress = AdaptiveProgress::new(total, stderr_draw_target(true), expanded_style);
    progress.bar.enable_steady_tick(TICK_INTERVAL);
    progress
}


pub struct ProgressReader<'a, R> {
    reader: R,
    progress: &'a AdaptiveProgress,
}


impl<'a, R> ProgressReader<'a, R> {
    pub fn new(reader: R, progress: &'a AdaptiveProgress) -> Self {
        Self { reader, progress }
    }
}


impl<R: Read> Read for ProgressReader<'_, R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.reader.read(buffer)?;
        self.progress.inc(read as u64);
        Ok(read)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;
    use parking_lot::Mutex;

    #[derive(Clone, Debug, Default)]
    struct CapturedTerm(Arc<Mutex<String>>);

    impl indicatif::TermLike for CapturedTerm {
        fn width(&self) -> u16 {
            80
        }

        fn move_cursor_up(&self, _n: usize) -> io::Result<()> {
            Ok(())
        }

        fn move_cursor_down(&self, _n: usize) -> io::Result<()> {
            Ok(())
        }

        fn move_cursor_right(&self, _n: usize) -> io::Result<()> {
            Ok(())
        }

        fn move_cursor_left(&self, _n: usize) -> io::Result<()> {
            Ok(())
        }

        fn write_line(&self, value: &str) -> io::Result<()> {
            self.write_str(value)?;
            self.write_str("\n")
        }

        fn write_str(&self, value: &str) -> io::Result<()> {
            self.0.lock().push_str(value);
            Ok(())
        }

        fn clear_line(&self) -> io::Result<()> {
            Ok(())
        }

        fn flush(&self) -> io::Result<()> {
            Ok(())
        }
    }

    fn visible_message_column(output: &str, message: &str) -> usize {
        let prefix = &output[..output.find(message).unwrap()];
        let mut escaped = false;
        prefix.chars().filter(|character| {
            if escaped {
                if *character == 'm' {
                    escaped = false;
                }
                false
            } else if *character == '\u{1b}' {
                escaped = true;
                false
            } else {
                true
            }
        }).count()
    }

    #[test]
    fn finished_status_message_aligns_with_the_spinner_message() {
        let active_output = CapturedTerm::default();
        let active = ProgressBar::with_draw_target(
            None,
            ProgressDrawTarget::term_like(Box::new(active_output.clone())),
        );
        active.set_style(spinner_style());
        active.set_message("Linking");
        active.tick();

        let finished_output = CapturedTerm::default();
        let finished = ProgressBar::with_draw_target(
            None,
            ProgressDrawTarget::term_like(Box::new(finished_output.clone())),
        );
        StatusLine {
            bar: finished,
            kind: StatusKind::Independent,
            finished: None,
        }.finish("Linked");

        let active_output = active_output.0.lock();
        let finished_output = finished_output.0.lock();
        let active_column = visible_message_column(&active_output, "Linking");
        assert_eq!(active_column, 2);
        assert_eq!(
            visible_message_column(&finished_output, "Linked"),
            active_column,
        );
    }


    #[test]
    fn progress_reader_counts_only_bytes_returned_by_reader() {
        let progress = byte_progress(Some(6));
        let mut reader = ProgressReader::new(&b"abcdef"[..], &progress);
        let mut buffer = [0; 4];

        assert_eq!(reader.read(&mut buffer).unwrap(), 4);
        assert_eq!(progress.bar.position(), 4);
        assert_eq!(reader.read(&mut buffer).unwrap(), 2);
        assert_eq!(progress.bar.position(), 6);
        assert_eq!(reader.read(&mut buffer).unwrap(), 0);
        assert_eq!(progress.bar.position(), 6);
    }

    #[test]
    fn nested_compilation_restores_the_outer_message() {
        let outer = StatusLine::start_compilation("outer", false);
        let inner = StatusLine::start_compilation("inner", true);

        assert_eq!(outer.kind, StatusKind::CompileOwner);
        assert!(matches!(inner.kind, StatusKind::Borrowed { .. }));
        assert_eq!(outer.bar.message(), "outer");

        inner.finish("ignored");
        assert_eq!(outer.bar.message(), "outer");
        assert!(ACTIVE_COMPILE_BAR.with(|active| active.borrow().is_some()));

        outer.finish("done");
        assert!(ACTIVE_COMPILE_BAR.with(|active| active.borrow().is_none()));
    }

    #[test]
    fn nested_status_never_leaves_a_finished_line() {
        let compile = StatusLine::start_compilation("Compiling", true);
        let link = StatusLine::start("Linking");

        assert!(matches!(link.kind, StatusKind::Borrowed { .. }));
        assert_eq!(compile.bar.message(), "Linking");
        link.finish("Linked");
        assert_eq!(compile.bar.message(), "Compiling");

        {
            let running = StatusLine::start("Running");
            assert_eq!(compile.bar.message(), "Running");
            drop(running);
        }
        assert_eq!(compile.bar.message(), "Compiling");
    }

    #[test]
    fn compilation_status_is_independent_between_threads() {
        let outer = StatusLine::start_compilation("outer", true);
        let thread_owned =
        std::thread::spawn(|| {
            StatusLine::start_compilation("thread", true).kind == StatusKind::CompileOwner
        })
            .join()
            .unwrap();

        assert!(thread_owned);
        assert!(ACTIVE_COMPILE_BAR.with(|active| active.borrow().is_some()));
        outer.finish("done");
    }

    #[test]
    fn dropped_compile_owner_releases_the_thread_bar() {
        {
            let status = StatusLine::start_compilation("compile", true);
            assert_eq!(status.kind, StatusKind::CompileOwner);
        }

        assert!(ACTIVE_COMPILE_BAR.with(|active| active.borrow().is_none()));
        let next = StatusLine::start_compilation("next", true);
        assert_eq!(next.kind, StatusKind::CompileOwner);
    }

    #[test]
    fn compilation_settles_after_the_borrowed_link_phase() {
        let compile = StatusLine::start_compilation("Compiling", true);
        let link = StatusLine::start("Linking");
        assert!(matches!(link.kind, StatusKind::Borrowed { .. }));

        link.finish("Linked");
        assert_eq!(compile.bar.message(), "Compiling");

        compile.finish("Built");
        assert!(ACTIVE_COMPILE_BAR.with(|active| active.borrow().is_none()));
    }


    #[test]
    fn progress_reader_does_not_count_failed_reads() {
        struct FailedReader;

        impl Read for FailedReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("read failed"))
            }
        }

        let progress = byte_progress(None);
        let error = ProgressReader::new(FailedReader, &progress)
            .read(&mut [0; 1])
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(progress.bar.position(), 0);
    }


    #[test]
    fn determinate_progress_starts_as_a_spinner_then_expands() {
        let output = CapturedTerm::default();
        let mut progress = AdaptiveProgress::new(
            Some(3),
            ProgressDrawTarget::term_like(Box::new(output.clone())),
            Some(bar_style(
                "{spinner:.green} {msg} {bar:40.cyan/blue} {pos}/{len} ({eta})",
            )),
        );
        progress.set_message("Removing 12 KiB");
        progress.inc(0);
        let initial = output.0.lock();
        assert!(initial.contains("Removing 12 KiB"));
        assert!(!initial.contains("0/3"));
        drop(initial);
        progress.started = Instant::now() - PROGRESS_BAR_DELAY;
        progress.inc(0);
        assert!(!output.0.lock().contains("0/3"));
        progress.inc(1);
        assert!(output.0.lock().contains("1/3"));
        progress.finish();
    }


    #[test]
    fn item_progress_tracks_units_and_message() {
        let progress = item_progress(3, "Removing 12 KiB");

        assert_eq!(progress.bar.length(), Some(3));
        assert_eq!(progress.bar.message(), "Removing 12 KiB");
        progress.inc(2);
        assert_eq!(progress.bar.position(), 2);
        progress.finish();
    }

}
