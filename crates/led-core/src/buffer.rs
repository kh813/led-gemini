use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use anyhow::Result;
use ropey::Rope;
use crate::{Encoding, LineEnding};
use encoding_rs::*;

pub struct Buffer {
    pub rope: Rope,
    pub path: Option<PathBuf>,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    pub modified: bool,
    pub read_only: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            path: None,
            encoding: Encoding::Utf8,
            line_ending: LineEnding::Lf,
            modified: false,
            read_only: false,
        }
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let (encoding, content) = Self::decode_bytes(&bytes);

        let line_ending = if content.contains("\r\n") {
            LineEnding::Crlf
        } else if content.contains('\r') {
            LineEnding::Cr
        } else {
            LineEnding::Lf
        };

        Ok(Self {
            rope: Rope::from_str(&content),
            path: Some(path),
            encoding,
            line_ending,
            modified: false,
            read_only: false,
        })
    }

    fn decode_bytes(bytes: &[u8]) -> (Encoding, String) {
        // Simple auto-detection: check for BOM first
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            return (Encoding::Utf8Bom, UTF_8.decode(&bytes[3..]).0.into_owned());
        }
        if bytes.starts_with(&[0xFF, 0xFE]) {
            return (Encoding::Utf16Le, UTF_16LE.decode(&bytes[2..]).0.into_owned());
        }
        if bytes.starts_with(&[0xFE, 0xFF]) {
            return (Encoding::Utf16Be, UTF_16BE.decode(&bytes[2..]).0.into_owned());
        }

        // Try UTF-8 first
        let (res, _enc, malformed) = UTF_8.decode(bytes);
        if !malformed {
            return (Encoding::Utf8, res.into_owned());
        }

        // Try Shift-JIS (common in Japan)
        let (res, _enc, malformed) = SHIFT_JIS.decode(bytes);
        if !malformed {
            return (Encoding::ShiftJis, res.into_owned());
        }

        // Fallback to UTF-8 (even if malformed)
        (Encoding::Utf8, UTF_8.decode(bytes).0.into_owned())
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = self.path.clone() {
            self.save_as(path)
        } else {
            anyhow::bail!("No path associated with buffer")
        }
    }

    pub fn save_as<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let content = self.rope.to_string();
        let content = match self.line_ending {
            LineEnding::Lf => content.replace("\r\n", "\n").replace('\r', "\n"),
            LineEnding::Crlf => content.replace("\r\n", "\n").replace('\r', "\n").replace('\n', "\r\n"),
            LineEnding::Cr => content.replace("\r\n", "\n").replace('\r', "\n").replace('\n', "\r"),
        };

        let encoder = match self.encoding {
            Encoding::Utf8 | Encoding::Utf8Bom => UTF_8,
            Encoding::Utf16Le => UTF_16LE,
            Encoding::Utf16Be => UTF_16BE,
            Encoding::ShiftJis => SHIFT_JIS,
            Encoding::EucJp => EUC_JP,
            Encoding::Iso2022Jp => ISO_2022_JP,
            Encoding::Latin1 => WINDOWS_1252,
        };

        let mut bytes = match self.encoding {
            Encoding::Utf8Bom => vec![0xEF, 0xBB, 0xBF],
            Encoding::Utf16Le => vec![0xFF, 0xFE],
            Encoding::Utf16Be => vec![0xFE, 0xFF],
            _ => vec![],
        };

        let (encoded_bytes, _, _malformed) = encoder.encode(&content);
        bytes.extend_from_slice(&encoded_bytes);

        let mut file = File::create(path.as_ref())?;
        file.write_all(&bytes)?;
        file.flush()?;

        self.path = Some(path.as_ref().to_path_buf());
        self.modified = false;
        Ok(())
    }
}
