use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn temporary_directory(prefix: &str) -> io::Result<PathBuf> {
    let base = std::env::temp_dir().canonicalize()?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    temporary_directory_at(&base, prefix, nonce)
}

fn temporary_directory_at(base: &Path, prefix: &str, nonce: u128) -> io::Result<PathBuf> {
    for attempt in 0..1024 {
        let path = base.join(format!("{prefix}-{}-{nonce}-{attempt}", std::process::id()));
        #[cfg(unix)]
        let result = {
            use std::os::unix::fs::DirBuilderExt;
            fs::DirBuilder::new().mode(0o700).create(&path)
        };
        #[cfg(not(unix))]
        let result = fs::create_dir(&path);
        match result {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "Unable to allocate an unused fixture directory",
    ))
}

#[cfg(unix)]
#[test]
fn temporary_directories_are_private_on_unix() {
    use std::os::unix::fs::PermissionsExt;

    let root = temporary_directory("cirnetrules-fixture-private-test").unwrap();
    let mode = fs::metadata(&root).unwrap().permissions().mode();
    fs::remove_dir(&root).unwrap();
    assert_eq!(mode & 0o077, 0, "fixture directory must be private");
}

#[test]
fn temporary_directories_are_distinct_when_clock_values_collide() {
    let root = temporary_directory("cirnetrules-fixture-collision-test").unwrap();
    let result = (|| -> io::Result<_> {
        let first = temporary_directory_at(&root, "fixture", 123)?;
        fs::write(first.join("sentinel"), b"preserve existing contents")?;
        let second = temporary_directory_at(&root, "fixture", 123)?;
        Ok((first.clone(), second, fs::read(first.join("sentinel"))?))
    })();
    fs::remove_dir_all(&root).unwrap();
    let (first, second, contents) = result.unwrap();
    assert_ne!(first, second);
    assert_eq!(contents, b"preserve existing contents");
}
