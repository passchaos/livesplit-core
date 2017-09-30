use std::borrow::Cow;
use std::io::Write;
use std::result::Result as StdResult;
use serde::ser::{Serialize, Serializer};
use {Run, Segment, Time, TimeSpan};
use time::formatter::{Complete, TimeFormatter};
use serde_json::{self, to_writer};
use comparison::best_split_times;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Json(error: serde_json::Error) {
            from()
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

struct SerTimeSpan(TimeSpan);

impl Serialize for SerTimeSpan {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&Complete.format(Some(self.0)))
    }
}

struct SerSplits<'a>(&'a [Segment]);

impl<'a> Serialize for SerSplits<'a> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.0.iter().map(|segment| {
            let title = segment.name();

            Split {
                title: if title.is_empty() { None } else { Some(title) },
                time: time(segment.personal_best_split_time()),
                best_time: time(segment.comparison(best_split_times::NAME)),
                best_segment: time(segment.best_segment_time()),
            }
        }))
    }
}

#[derive(Serialize)]
struct Splits<'a> {
    #[serde(skip_serializing_if = "Option::is_none")] title: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")] attempt_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")] start_delay: Option<SerTimeSpan>,
    splits: SerSplits<'a>,
}

#[derive(Serialize)]
struct Split<'a> {
    #[serde(skip_serializing_if = "Option::is_none")] title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")] time: Option<SerTimeSpan>,
    #[serde(skip_serializing_if = "Option::is_none")] best_time: Option<SerTimeSpan>,
    #[serde(skip_serializing_if = "Option::is_none")] best_segment: Option<SerTimeSpan>,
}

fn time_span(t: TimeSpan) -> Option<SerTimeSpan> {
    if t == TimeSpan::zero() {
        None
    } else {
        Some(SerTimeSpan(t))
    }
}

fn time(t: Time) -> Option<SerTimeSpan> {
    t.real_time.and_then(time_span)
}

/// Recalculates the Best Split Times, so they can get saved properly.
pub fn save_with_recalculation<W: Write>(run: &mut Run, writer: W) -> Result<()> {
    // TODO Assumes the Best Split Times Comparison Generator is part of the
    // run's comparison generators. There's no public API for changing them yet.
    run.regenerate_comparisons();

    save(run, writer)
}

/// Best Split Times only get stored correctly if the Comparison Generator has been executed.
pub fn save<W: Write>(run: &Run, writer: W) -> Result<()> {
    let title = run.extended_name(false);
    let attempt_count = run.attempt_count();

    to_writer(
        writer,
        &Splits {
            title: if title.is_empty() { None } else { Some(title) },
            attempt_count: if attempt_count == 0 {
                None
            } else {
                Some(attempt_count)
            },
            start_delay: time_span(-run.offset()),
            splits: SerSplits(run.segments()),
        },
    )?;

    Ok(())
}
