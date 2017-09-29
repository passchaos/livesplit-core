use std::io::{self, Write};
use std::result::Result as StdResult;
use {Run, base64};
use super::file_write::FileWrite;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(error: io::Error) {
            from()
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

pub fn save<W: FileWrite>(run: &Run, mut writer: W) -> Result<()> {
    let mut buf = Vec::new();

    writeln!(writer, "Title={}", run.extended_name(false))?;
    writeln!(writer, "Attempts={}", run.attempt_count())?;
    writeln!(writer, "Offset={}", -run.offset().total_milliseconds())?;
    writeln!(writer, "Size=152,25")?;

    for segment in run.segments() {
        writeln!(
            writer,
            "{name},{old_time},{pb_time},{best_time}",
            name = segment.name(),
            old_time = segment
                .comparison("Old Run")
                .real_time
                .map_or(0.0, |t| t.total_seconds()),
            pb_time = segment
                .personal_best_split_time()
                .real_time
                .map_or(0.0, |t| t.total_seconds()),
            best_time = segment
                .best_segment_time()
                .real_time
                .map_or(0.0, |t| t.total_seconds()),
        )?;
    }

    write!(writer, "Icons=")?;
    for (i, segment) in run.segments().iter().enumerate() {
        if i != 0 {
            write!(writer, ",")?;
        }
        if !writer.supports_other_files() || !segment.icon().url().starts_with("data:;base64,") {
            write!(writer, r#""""#)?;
        } else {
            write!(writer, r#""icon{}""#, i)?;
        }
    }
    writeln!(writer)?;

    if writer.supports_other_files() {
        for (i, segment) in run.segments().iter().enumerate() {
            // TODO guess file extension
            let file = writer.start_other_file(&format!(r#""icon{}""#, i))?;
            let url = segment.icon().url();
            if url.starts_with("data:;base64,") {
                let src = &url["data:;base64,".len()..];
                buf.clear();
                if base64::decode_config_buf(src, base64::STANDARD, &mut buf).is_ok() {
                    file.write_all(&buf)?;
                }
            }
        }
    }

    Ok(())
}
