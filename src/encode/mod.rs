mod job;
mod output;
mod sample_entry;
mod transcode;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::task::JoinSet;

use crate::codec::EncodeRecipe;
use crate::error::{Error, Result};
use crate::input::ensure_mp4_input;

pub use job::EncodeJob;
pub use output::default_output_path;

/// 1 ジョブ分のエンコード進捗を追跡する
///
/// デコード完了後に全体のフレーム数が分かるので `total` にセットし、
/// エンコードループ内で `processed` をインクリメントしていく。
/// UI 側は定期的に `ratio()` をポーリングして進捗バーを更新する。
pub struct JobProgress {
    processed: AtomicU64,
    total: AtomicU64,
    notify: Arc<tokio::sync::Notify>,
}

impl Default for JobProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl JobProgress {
    pub fn new() -> Self {
        Self {
            processed: AtomicU64::new(0),
            total: AtomicU64::new(0),
            notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// 通知ハンドルを返す (UI 側のイベント駆動待機用)
    pub fn notify_handle(&self) -> Arc<tokio::sync::Notify> {
        self.notify.clone()
    }

    /// 総フレーム数をセットする (demux 完了後など)
    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
        self.notify.notify_one();
    }

    /// 処理済みフレーム数を加算する
    pub fn add_processed(&self, count: u64) {
        self.processed.fetch_add(count, Ordering::Relaxed);
        self.notify.notify_one();
    }

    /// 進捗割合 (0.0 - 1.0) を返す
    pub fn ratio(&self) -> f64 {
        let total = self.total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let processed = self.processed.load(Ordering::Relaxed);
        (processed as f64 / total as f64).min(1.0)
    }
}

pub async fn encode_file_async(
    input: &Path,
    output: &Path,
    recipe: EncodeRecipe,
    overwrite: bool,
    progress: Arc<JobProgress>,
) -> Result<()> {
    ensure_mp4_input(input)?;
    ensure_mp4_output(output)?;
    let input = input.to_path_buf();
    let output = output.to_path_buf();
    tokio::task::spawn_blocking(move || encode_mp4_to_mp4_sync(&input, &output, recipe, overwrite, &progress))
        .await
        .expect("encode join failed")
}

pub async fn run_jobs_async(
    jobs: Vec<EncodeJob>,
    progresses: Vec<Arc<JobProgress>>,
) -> Vec<(EncodeJob, Result<()>)> {
    let mut set = JoinSet::new();
    for (job, progress) in jobs.into_iter().zip(progresses) {
        set.spawn(async move {
            let result = encode_file_async(&job.input, &job.output, job.recipe, job.overwrite, progress).await;
            (job, result)
        });
    }
    let mut out = Vec::new();
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(pair) => out.push(pair),
            Err(e) => {
                tracing::error!(error = %e, "encode task panicked");
                out.push((
                    EncodeJob::new(PathBuf::new(), PathBuf::new(), EncodeRecipe::default(), false),
                    Err(Error::Message(format!("encode task panicked: {e}"))),
                ));
            }
        }
    }
    out
}

fn ensure_mp4_output(path: &Path) -> Result<()> {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("mp4"))
    {
        Ok(())
    } else {
        Err(Error::Message(format!(
            "output must be .mp4: {}",
            path.display()
        )))
    }
}

fn encode_mp4_to_mp4_sync(
    input: &Path,
    output: &Path,
    recipe: EncodeRecipe,
    overwrite: bool,
    progress: &JobProgress,
) -> Result<()> {
    transcode::transcode(input, output, recipe, overwrite, progress)
}
