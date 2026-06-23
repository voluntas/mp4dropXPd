use std::path::{Path, PathBuf};

use crate::codec::EncodeRecipe;

pub fn default_output_path(input: &Path, recipe: EncodeRecipe) -> PathBuf {
    let stem = input
        .file_stem()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    let parent = input.parent().unwrap_or(Path::new("."));
    let tag = recipe.codec_tag();
    parent.join(format!(
        "{}.{}.mp4",
        stem.to_string_lossy(),
        tag
    ))
}
