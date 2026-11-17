use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const FONT_SIZE_KEY: &str = "terminal_font_size_points";
static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalFontSize(u16);

impl TerminalFontSize {
    pub const MIN_POINTS: u16 = 6;
    pub const MAX_POINTS: u16 = 72;

    pub fn new(points: i64) -> io::Result<Self> {
        if (i64::from(Self::MIN_POINTS)..=i64::from(Self::MAX_POINTS)).contains(&points) {
            return Ok(Self(points as u16));
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Terminal font size must be between {} and {} points",
                Self::MIN_POINTS,
                Self::MAX_POINTS
            ),
        ))
    }

    pub fn points(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub window_width: i32,
    pub window_height: i32,
    pub terminal_font_size: Option<TerminalFontSize>,
    // Keep keys owned by alternate or newer frontends when this setting changes.
    document: toml::Table,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window_width: 800,
            window_height: 500,
            terminal_font_size: None,
            document: toml::Table::new(),
        }
    }
}

impl Settings {
    pub fn config_path() -> io::Result<PathBuf> {
        let config_dir = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .filter(|path| path.is_absolute())
                    .map(|path| path.join(".config"))
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Set XDG_CONFIG_HOME or HOME to an absolute path to locate mytthy settings",
                )
            })?;

        Ok(config_dir.join("mytthy/settings.toml"))
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(path_error(path, "read", error)),
        };
        Self::parse(&contents).map_err(|error| path_error(path, "read", error))
    }

    fn parse(contents: &str) -> io::Result<Self> {
        let document = contents
            .parse::<toml::Table>()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let terminal_font_size = document
            .get(FONT_SIZE_KEY)
            .map(|value| {
                let points = value.as_integer().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Terminal font size must be a whole number of points",
                    )
                })?;
                TerminalFontSize::new(points)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
            })
            .transpose()?;

        Ok(Self {
            terminal_font_size,
            document,
            ..Self::default()
        })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut document = self.document.clone();
        match self.terminal_font_size {
            Some(size) => {
                document.insert(
                    FONT_SIZE_KEY.into(),
                    toml::Value::Integer(i64::from(size.points())),
                );
            }
            None => {
                document.remove(FONT_SIZE_KEY);
            }
        }

        write_atomically(path, document.to_string().as_bytes())
            .map_err(|error| path_error(path, "save", error))
    }
}

fn path_error(path: &Path, operation: &str, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!(
            "Could not {operation} settings at {}: {error}",
            path.display()
        ),
    )
}

fn create_temp_file(dir: &Path) -> io::Result<(PathBuf, File)> {
    loop {
        let id = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let path = dir.join(format!(".mytthy-settings-{}-{id}.tmp", std::process::id()));
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
}

fn write_atomically(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let (temp_path, mut file) = create_temp_file(parent)?;
    let written = file.write_all(contents).and_then(|()| file.sync_all());
    drop(file);
    let result = written.and_then(|()| fs::rename(&temp_path, path));
    if result.is_err() {
        let _ = fs::remove_file(temp_path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            loop {
                let id = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
                let path =
                    std::env::temp_dir().join(format!("mytthy-test-{}-{id}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(error) => panic!("could not create test directory: {error}"),
                }
            }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn keeps_defaults_without_a_settings_file() {
        let dir = TestDir::new();
        let settings = Settings::load(&dir.0.join("settings.toml")).unwrap();
        assert_eq!(settings.terminal_font_size, None);
        assert_eq!((settings.window_width, settings.window_height), (800, 500));
    }

    #[test]
    fn saves_and_resets_font_size_while_preserving_other_keys() {
        let dir = TestDir::new();
        let path = dir.0.join("settings.toml");
        fs::write(&path, "future_setting = true\n").unwrap();

        let mut settings = Settings::load(&path).unwrap();
        settings.terminal_font_size = Some(TerminalFontSize::new(16).unwrap());
        settings.save(&path).unwrap();
        let mut settings = Settings::load(&path).unwrap();
        assert_eq!(settings.terminal_font_size.unwrap().points(), 16);

        settings.terminal_font_size = None;
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap().terminal_font_size, None);
        assert_eq!(
            Settings::load(&path).unwrap().document["future_setting"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn reports_invalid_font_size_and_parse_errors_without_editing_file() {
        let dir = TestDir::new();
        let path = dir.0.join("settings.toml");
        for invalid in ["5", "73", "12.5", "true", "["] {
            let contents = format!("{FONT_SIZE_KEY} = {invalid}");
            fs::write(&path, &contents).unwrap();
            let error = Settings::load(&path).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert!(error.to_string().contains(path.to_str().unwrap()));
            assert_eq!(fs::read_to_string(&path).unwrap(), contents);
        }
    }

    #[test]
    fn surfaces_save_failure_and_removes_temporary_file() {
        let dir = TestDir::new();
        let path = dir.0.join("settings.toml");
        fs::create_dir(&path).unwrap();

        assert!(Settings::default().save(&path).is_err());
        assert!(path.is_dir());
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
    }
}
