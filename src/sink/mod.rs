pub mod jsonl_sink;

pub use jsonl_sink::JsonLinesSink;

use anyhow::Result;

pub trait ConsumerSink<T> {
    fn consume(&mut self, item: &T) -> Result<()>;
}
