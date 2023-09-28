use std::io::Read;
use std::time::{Duration, Instant};

/// Smoothing factor
const ALPHA: f64 = 0.5;

/// Minimum update rate for the moving average to avoid instability due to too short read times
const UPDATE_RATE: Duration = Duration::from_millis(10);

#[derive(Debug)]
struct Buffer {
    time: Instant,
    read: usize,
}

impl Buffer {
    fn new() -> Self {
        Self {
            time: Instant::now(),
            read: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Average(f64);

impl Average {
    pub fn bytes_per_second(self) -> f64 {
        self.0
    }

    pub fn kilobytes_per_second(self) -> f64 {
        self.0 / 1_000.0
    }

    pub fn megabytes_per_second(self) -> f64 {
        self.0 / 1_000_000.0
    }
}

#[derive(Debug)]
pub struct MeasuringReader<R> {
    /// Wrapped reader
    inner: R,
    size_hint: Option<usize>,
    /// Total amount of bytes read
    total: usize,
    avg: f64,
    buf: Buffer,
}

impl<R> MeasuringReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            size_hint: None,
            total: 0,
            avg: 0.0,
            buf: Buffer::new(),
        }
    }

    pub fn with_size_hint(inner: R, size_hint: usize) -> Self {
        Self {
            inner,
            size_hint: Some(size_hint),
            total: 0,
            avg: 0.0,
            buf: Buffer::new(),
        }
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn avg(&self) -> Average {
        Average(self.avg)
    }

    /// Time remaining
    ///
    /// Returns `None` if `size_hint` was not set or a remaining time could not be computed.
    pub fn time_remaining(&self) -> Option<Duration> {
        let Some(size_hint) = self.size_hint else {
            return None;
        };

        Duration::try_from_secs_f64((size_hint - self.total) as f64 / self.avg).ok()
    }

    pub fn percentage(&self) -> Option<f64> {
        let Some(size_hint) = self.size_hint else {
            return None;
        };

        Some((self.total as f64 / size_hint as f64) * 100.0)
    }
}

impl<R: Read> Read for MeasuringReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf)?;

        let elapsed = self.buf.time.elapsed();
        self.buf.read += read;

        // Update the average speed if the minimum update rate has been surpassed
        if elapsed >= UPDATE_RATE {
            let speed = self.buf.read as f64 / elapsed.as_secs_f64();
            self.avg = ALPHA * speed + (1.0 - ALPHA) * self.avg;
            self.buf = Buffer::new();
        }

        // Update total bytes read
        self.total += read;

        Ok(read)
    }
}
