use std::io::{Result as IoResult, Write};
use std::fs::File;

/// Abstracts writing out multiple files. Theoretical implementations could
/// include only writing out the main data, writing out everything as individual
/// files, or storing all the files in a single file, like an archive.
pub trait FileWrite: Write {
    type OtherFile: Write;
    fn supports_other_files(&self) -> bool {
        true
    }
    /// The main file may not be valid anymore after opening another file.
    fn start_other_file(&mut self, name: &str) -> IoResult<&mut Self::OtherFile>;
}

/// Stores only the main file in the inner Writer.
#[derive(From)]
pub struct OnlyStoreMainFile<W>(pub W);

impl<W: Write> FileWrite for OnlyStoreMainFile<W> {
    type OtherFile = Self;
    fn supports_other_files(&self) -> bool {
        false
    }
    fn start_other_file(&mut self, _: &str) -> IoResult<&mut Self::OtherFile> {
        panic!("Doesn't support other files");
    }
}

impl<W: Write> Write for OnlyStoreMainFile<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> IoResult<()> {
        self.0.flush()
    }
}

/// Stores all the other files as actual files on the file system.
pub struct StoreOtherFiles<W>(W, Option<File>);

impl<W: Write> FileWrite for StoreOtherFiles<W> {
    type OtherFile = File;
    fn start_other_file(&mut self, name: &str) -> IoResult<&mut Self::OtherFile> {
        self.1 = Some(File::create(name)?);
        Ok(self.1.as_mut().unwrap())
    }
}

impl<W: Write> Write for StoreOtherFiles<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> IoResult<()> {
        self.0.flush()
    }
}
