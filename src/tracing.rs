//! Provides convenient tracing setup.
//!
//! # Examples
//!
//! Set up tracing for stdout and file logging
//! ```
//! utility::tracing::stdout().with_file("log.txt").init();
//! ```
//!
//! Standard log level is INFO, use [`with_level`] to set another minimum log level
//! ```
//! utility::tracing::file("log.txt").with_level(tracing::Level::DEBUG).init();
//! ```
//!
//! Set up tracing for use with tracy
//! ```
//! let _defer = utility::tracing::tracy().init();
//! ```

use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;

/// Amount to time to wait for tracy to establish / finish the connection
const TRACY_CONNECTION_TIMEOUT: Duration = Duration::from_secs(1);

/// Configuration to use for tracing
#[must_use]
pub struct TracingBuilder {
    log_to_stdout: bool,
    log_to_tracy: bool,
    log_to_file: Option<PathBuf>,
    log_level: Option<tracing::Level>,
}

impl TracingBuilder {
    /// Create an empty instance with no logging enabled
    pub fn empty() -> Self {
        Self {
            log_to_stdout: false,
            log_to_tracy: false,
            log_to_file: None,
            log_level: None,
        }
    }

    /// Enable logging to stdout
    pub fn with_stdout(mut self) -> Self {
        self.log_to_stdout = true;
        self
    }

    /// Enable logging to tracy
    #[cfg(feature = "tracy")]
    pub fn with_tracy(mut self) -> Self {
        self.log_to_tracy = true;
        self
    }

    /// Enable logging to the specified file
    pub fn with_file(mut self, path: impl AsRef<Path>) -> Self {
        self.log_to_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the log level
    pub fn with_level(mut self, level: tracing::Level) -> Self {
        self.log_level = Some(level);
        self
    }

    /// Initialize the tracing configuration
    ///
    /// Returns [`TracingDefer`], which can be ignored if the `tracy` feature is not enabled.
    pub fn init(self) -> TracingDefer {
        // stdout
        let stdout_layer = self
            .log_to_stdout
            .then(|| tracing_subscriber::fmt::layer().with_ansi(false).with_writer(std::io::stdout));

        // file
        let file_layer = self.log_to_file.map(|path| {
            // Create file and it's writer
            let log_file = std::fs::File::create(path).expect("failed to create log file");
            let wtr = Writer(Arc::new(Mutex::new(BufWriter::new(log_file))));

            // Register a panic hook that prints the stack backtrace to the file
            let panic_wtr = wtr.clone();
            let old_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                if let Ok(mut guard) = panic_wtr.0.try_lock() {
                    let backtrace = std::backtrace::Backtrace::force_capture();
                    let msg = format!("{info}\n\nStack backtrace:\n{backtrace}");
                    guard
                        .write_all(msg.as_bytes())
                        .expect("failed to write backtrace");
                    guard.flush().expect("failed to flush buffer");
                }
                old_hook(info);
            }));

            tracing_subscriber::fmt::layer()
                .with_writer(wtr)
                .with_ansi(false)
                .pretty()
        });

        // tracy
        #[cfg(feature = "tracy")]
        let tracy_layer = self.log_to_tracy.then(tracing_tracy::TracyLayer::default);
        #[cfg(not(feature = "tracy"))]
        let tracy_layer: Option<tracing_subscriber::fmt::Layer<_>> = None;

        // Min log level
        let min_level = self.log_level.unwrap_or(tracing::Level::INFO);

        // Combine layers
        let subscriber = tracing_subscriber::registry()
            .with(stdout_layer)
            .with(tracy_layer)
            .with(file_layer)
            .with(tracing::level_filters::LevelFilter::from_level(min_level));

        // Register the subscriber
        tracing::subscriber::set_global_default(subscriber)
            .expect("failed to set global tracing subscriber");

        TracingDefer::new(self.log_to_tracy)
    }
}

impl Default for TracingBuilder {
    /// Create a default instance with logging to stdout enabled
    #[inline]
    fn default() -> Self {
        TracingBuilder::empty().with_stdout()
    }
}

/// Create [`TracingBuilder`] instance with logging to stdout enabled
///
/// Short form of `TracingBuilder::empty().with_stdout()`
#[inline]
pub fn stdout() -> TracingBuilder {
    TracingBuilder::empty().with_stdout()
}

/// Create an [`TracingBuilder`] instance with logging to tracy enabled
///
/// Short form of `TracingBuilder::empty().with_tracy()`
#[cfg(feature = "tracy")]
#[inline]
pub fn tracy() -> TracingBuilder {
    TracingBuilder::empty().with_tracy()
}

/// Create an [`TracingBuilder`] instance with logging to a file enabled
///
/// Short form of `TracingBuilder::empty().with_with(<path>)`
#[inline]
pub fn file(path: impl AsRef<Path>) -> TracingBuilder {
    TracingBuilder::empty().with_file(path)
}

/// Utility struct that ensures proper shutdown of tracy when dropped
#[cfg_attr(feature = "tracy", must_use)]
pub struct TracingDefer {
    is_tracy_enabled: bool,
}

impl TracingDefer {
    fn new(is_tracy_enabled: bool) -> Self {
        if is_tracy_enabled {
            std::thread::sleep(TRACY_CONNECTION_TIMEOUT);
        }
        Self { is_tracy_enabled }
    }
}

impl Drop for TracingDefer {
    fn drop(&mut self) {
        if self.is_tracy_enabled {
            std::thread::sleep(TRACY_CONNECTION_TIMEOUT);
        }
    }
}

/// New type that implements Write for a MutexGuard
struct MutexWriteGuard<'a, W>(MutexGuard<'a, W>);

impl<W> Write for MutexWriteGuard<'_, W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes_written = self.0.write(buf)?;
        self.flush()?;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

/// New type that implements Write for a Arc<Mutex<W>>
struct Writer<W>(Arc<Mutex<W>>);

impl<'a, W> tracing_subscriber::fmt::MakeWriter<'a> for Writer<W>
where
    W: Write + 'a,
{
    type Writer = MutexWriteGuard<'a, W>;

    fn make_writer(&'a self) -> Self::Writer {
        MutexWriteGuard(self.0.lock().expect("lock poisoned"))
    }
}

impl<W> Clone for Writer<W> {
    fn clone(&self) -> Self {
        Writer(Arc::clone(&self.0))
    }
}
