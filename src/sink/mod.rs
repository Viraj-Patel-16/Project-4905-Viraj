pub mod console_sink;
pub mod jsonl_sink;

pub use console_sink::ConsoleSink;
pub use jsonl_sink::JsonLinesSink;

use anyhow::Result;

pub trait ConsumerSink<T> {
    fn consume(&mut self, item: &T) -> Result<()>;
}
