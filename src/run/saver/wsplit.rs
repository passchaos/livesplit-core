use std::io::{self, Write};
use std::result::Result as StdResult;
use Run;
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
        let icon = segment.icon();
        if !writer.supports_other_files() || icon.url().is_empty() {
            write!(writer, r#""""#)?;
        } else {
            write!(writer, r#""icon{}""#, i)?;
            if let Some(extension) = icon.file_extension() {
                write!(writer, ".{}", extension)?;
            }
        }
    }
    writeln!(writer)?;

    if writer.supports_other_files() {
        for (i, segment) in run.segments().iter().enumerate() {
            let icon = segment.icon();
            let mut name = format!(r#""icon{}""#, i);
            if let Some(extension) = icon.file_extension() {
                name.push('.');
                name.push_str(extension);
            }
            let file = writer.start_other_file(&name)?;
            if icon.decode_data(&mut buf).is_ok() {
                file.write_all(&buf)?;
            }
        }
    }

    Ok(())
}
