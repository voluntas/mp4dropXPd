use std::path::Path;

use crate::error::{Error, Result};

/// 入力として受け付けるのは MP4 ファイルのみ（拡張子 `.mp4`、大文字小文字無視）
pub fn is_mp4_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("mp4"))
}

pub fn filter_mp4_paths(paths: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
    paths.iter().filter(|p| is_mp4_file(p)).cloned().collect()
}

pub fn ensure_mp4_input(path: &Path) -> Result<()> {
    if is_mp4_file(path) {
        Ok(())
    } else {
        Err(Error::UnsupportedFormat {
            path: path.to_path_buf(),
        })
    }
}
