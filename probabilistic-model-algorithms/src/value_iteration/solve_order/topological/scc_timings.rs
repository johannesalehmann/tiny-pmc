use crate::value_iteration::solve_order::ModelSize;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub trait TopoTiming {
    type Entry;
    fn start_entry(&self, size: &ModelSize) -> Self::Entry;
    fn finish_entry(&mut self, entry: Self::Entry);
    fn write_topo_timings(&self);
}

impl TopoTiming for () {
    type Entry = ();

    fn start_entry(&self, _size: &ModelSize) -> Self::Entry {
        ()
    }

    fn finish_entry(&mut self, _entry: Self::Entry) {}

    fn write_topo_timings(&self) {}
}

#[derive(Clone, Debug)]
pub enum SccTimingOutput {
    Stdout,
    File(PathBuf),
}

pub struct SccTimings {
    output: SccTimingOutput,
    entries: Vec<SccTiming>,
}

impl SccTimings {
    pub fn new(output: SccTimingOutput) -> Self {
        Self {
            output,
            entries: Vec::new(),
        }
    }

    fn write_entries(&self, out: &mut impl Write) {
        out.write_all("{ \"entries\": [\n".as_bytes()).unwrap();
        for (i, entry) in self.entries.iter().enumerate() {
            out.write_all(
                format!(
                    "{{ \"states\": {}, \"choices\": {}, \"branches\": {}, \"elapsed\": {} }}{}\n",
                    entry.size.states,
                    entry.size.choices,
                    entry.size.branches,
                    entry.elapsed.as_secs_f64(),
                    if i + 1 < self.entries.len() { "," } else { "" }
                )
                .as_bytes(),
            )
            .unwrap();
        }
        out.write_all("]}\n".as_bytes()).unwrap();
    }
}

impl TopoTiming for SccTimings {
    type Entry = (Instant, ModelSize);

    fn start_entry(&self, size: &ModelSize) -> Self::Entry {
        (Instant::now(), size.clone())
    }

    fn finish_entry(&mut self, (start, size): Self::Entry) {
        self.entries.push(SccTiming {
            size,
            elapsed: start.elapsed(),
        })
    }

    fn write_topo_timings(&self) {
        match &self.output {
            SccTimingOutput::Stdout => self.write_entries(&mut std::io::stdout().lock()),
            SccTimingOutput::File(path) => self.write_entries(
                &mut File::create(path).expect("Failed to create file for SCC timings"),
            ),
        }
    }
}

struct SccTiming {
    size: ModelSize,
    elapsed: Duration,
}
