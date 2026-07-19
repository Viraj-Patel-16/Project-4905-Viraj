use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use super::ConsumerSink;

pub struct JsonLinesSink {
    writer: BufWriter<File>,
}

impl JsonLinesSink {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

impl<T> ConsumerSink<T> for JsonLinesSink
where
    T: Serialize,
{
    fn consume(&mut self, item: &T) -> Result<()> {
        serde_json::to_writer(&mut self.writer, item)?;
        writeln!(self.writer)?;
        Ok(())
    }
}
