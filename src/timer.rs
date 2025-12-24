use web_time::{Duration, Instant};

/// Keeps track of the duration of each frame to record delta time and an estimate of the current
/// FPS.
pub struct FrameTimer {
    /// The instant the previous frame was completed.
    last_frame: Instant,
    /// The duration of how long the last frame took to complete.
    dt: Duration,

    /// The instant of the last full second of updates.
    last_second: Instant,
    /// The number of frames accumulated since the `last_second`.
    frames_accumulated: usize,
    /// An estimate of the current FPS in the last second.
    fps: f32,
}

impl FrameTimer {
    /// Creates a new [`FrameTimer`].
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            dt: Duration::from_secs(0),
            last_second: Instant::now(),
            frames_accumulated: 0,
            fps: 0.0,
        }
    }

    /// Advances the timer by one frame.
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.dt = self.last_frame.elapsed();
        self.last_frame = now;

        self.frames_accumulated += 1;

        let last_second_elapsed = self.last_second.elapsed();

        if last_second_elapsed > Duration::from_secs(1) {
            self.fps = self.frames_accumulated as f32 / last_second_elapsed.as_secs_f32();

            log::info!(
                "Running at {:.2} fps ({} frames accumulated)",
                self.fps,
                self.frames_accumulated
            );

            self.last_second = now;
            self.frames_accumulated = 0;
        }
    }
}
