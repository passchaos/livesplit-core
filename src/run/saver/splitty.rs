use std::io::Write;
use std::result::Result as StdResult;
use serde::ser::{Serialize, Serializer};
use {Run, Segment, Time, TimeSpan};
use serde_json::{self, to_writer};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Json(error: serde_json::Error) {
            from()
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

struct SerSplits<'a>(&'a [Segment]);

impl<'a> Serialize for SerSplits<'a> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut last_split_time = 0.0;

        serializer.collect_seq(self.0.iter().map(|segment| {
            let pb_split = time(segment.personal_best_split_time());
            let pb_duration = pb_split.map(|t| {
                let res = t - last_split_time;
                last_split_time = t;
                res
            });

            Split {
                name: segment.name(),
                pb_split,
                pb_duration,
                split_best: time(segment.best_segment_time()),
            }
        }))
    }
}

#[derive(Serialize)]
struct Splits<'a> {
    timer_name: &'a str,
    run_name: &'a str,
    hash: &'static str,
    start_delay: f64,
    run_count: u32,
    splits: SerSplits<'a>,
    timer_type: u8,
}

#[derive(Serialize)]
struct Split<'a> {
    name: &'a str,
    pb_split: Option<f64>,
    pb_duration: Option<f64>,
    split_best: Option<f64>,
}

fn time_span(t: TimeSpan) -> f64 {
    t.total_milliseconds()
}

fn time(t: Time) -> Option<f64> {
    t.real_time.map(time_span)
}

pub fn save<W: Write>(run: &Run, writer: W) -> Result<()> {
    let title = run.extended_name(false);

    // TODO Implement saving Game Time instead of Real Time

    to_writer(
        writer,
        &Splits {
            timer_name: &title,
            run_name: &title,
            hash: "",
            start_delay: -time_span(run.offset()),
            run_count: run.attempt_count(),
            splits: SerSplits(run.segments()),
            timer_type: 0,
        },
    )?;

    Ok(())
}
