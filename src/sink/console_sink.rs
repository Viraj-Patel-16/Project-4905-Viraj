use std::fmt::Debug;

use anyhow::Result;

use super::ConsumerSink;

pub struct ConsoleSink;

impl ConsoleSink {
    pub fn new() -> Self {
        Self
    }
}

impl<T> ConsumerSink<T> for ConsoleSink
where
    T: Debug,
{
    fn consume(&mut self, item: &T) -> Result<()> {
        println!("{:#?}", item);
        Ok(())
    }
}
