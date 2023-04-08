use std::path::{Path, PathBuf};
use tracing_subscriber::layer::SubscriberExt;

#[must_use]
pub struct Tracing {
    log_to_stdout: bool,
    #[cfg(feature = "tracy")]
    log_to_tracy: bool,
    log_to_file: Option<PathBuf>,
}

impl Tracing {
    /// Create an instance with logging to stdout enabled
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty instance with no logging enabled
    pub fn empty() -> Self {
        Self {
            log_to_stdout: false,
            #[cfg(feature = "tracy")]
            log_to_tracy: false,
            log_to_file: None,
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

    /// Initialize the tracing configuration
    pub fn init(self) {
        // stdout
        let stdout_layer = self
            .log_to_stdout
            .then(|| tracing_subscriber::fmt::layer().with_writer(std::io::stdout));

        // file
        let file_layer = self.log_to_file.map(|path| {
            let log_file = std::fs::File::create(path).expect("failed to create log file");
            tracing_subscriber::fmt::layer()
                .with_writer(std::sync::Mutex::new(log_file))
                .with_ansi(false)
                .pretty()
        });

        // tracy
        #[cfg(feature = "tracy")]
        let tracy_layer = self.log_to_tracy.then(tracing_tracy::TracyLayer::new);
        #[cfg(not(feature = "tracy"))]
        let tracy_layer: Option<tracing_subscriber::fmt::Layer<_>> = None;

        // Combine layers
        let subscriber = tracing_subscriber::registry()
            .with(stdout_layer)
            .with(tracy_layer)
            .with(file_layer);

        // Register the subscriber
        tracing::subscriber::set_global_default(subscriber)
            .expect("failed to set global tracing subscriber");
    }
}

impl Default for Tracing {
    /// Create a default instance with logging to stdout enabled
    fn default() -> Self {
        Self {
            log_to_stdout: true,
            #[cfg(feature = "tracy")]
            log_to_tracy: false,
            log_to_file: None,
        }
    }
}
