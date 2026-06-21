use std::path::PathBuf;

use crate::codec::EncodeRecipe;

#[derive(Debug, Clone)]
pub struct EncodeJob {
    pub input: PathBuf,
    pub output: PathBuf,
    pub recipe: EncodeRecipe,
}

impl EncodeJob {
    pub fn new(input: PathBuf, output: PathBuf, recipe: EncodeRecipe) -> Self {
        Self {
            input,
            output,
            recipe,
        }
    }
}
