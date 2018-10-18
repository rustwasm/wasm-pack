use slog::Logger;

// Create a logger that ignores log messages for testing.
pub fn null_logger() -> Logger {
    Logger::root(slog::Discard, o!())
}
