use std::path::Path;
use std::collections::BTreeMap;
use {Class, Result};
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use jni_cpp;

mod jna;
mod jni;
mod wasm;

pub fn write<P: AsRef<Path>>(path: P, classes: &BTreeMap<String, Class>) -> Result<()> {
    let mut path = path.as_ref().to_owned();

    path.push("jna");
    create_dir_all(&path)?;
    jna::write(&path, classes)?;
    path.pop();

    path.push("jni");
    create_dir_all(&path)?;
    jni::write(&path, classes)?;
    path.pop();

    path.push("LiveSplitCoreJNI.cpp");
    jni_cpp::write(BufWriter::new(File::create(&path)?), classes)?;
    path.pop();

    path.push("wasm");
    create_dir_all(&path)?;
    wasm::write(&path, classes)?;
    path.pop();

    Ok(())
}

fn write_class_comments<W: Write>(mut writer: W, comments: &[String]) -> Result<()> {
    write!(
        writer,
        r#"
/**"#
    )?;

    for comment in comments {
        write!(
            writer,
            r#"
 * {}"#,
            comment
                .replace("<NULL>", "null")
                .replace("<TRUE>", "true")
                .replace("<FALSE>", "false")
        )?;
    }

    write!(
        writer,
        r#"
 */"#
    )
}
