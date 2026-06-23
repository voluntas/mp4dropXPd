use std::io::{Seek, SeekFrom, Write};
use std::num::NonZeroU32;
use std::path::Path;

use shiguredo_mp4::TrackKind;
use shiguredo_mp4::mux::{Mp4FileMuxer, Sample as MuxSample};

use crate::error::{Error, Result};

use super::{AudioOutput, EncodedAudioSample, EncodedVideoSample, OutputKind, VideoOutput};
struct TempFile {
    path: std::path::PathBuf,
    committed: bool,
}

#[cfg(target_os = "macos")]
impl TempFile {
    /// `output` と同じディレクトリ内に pid + counter を含む
    /// 一意な一時ファイルパスを生成して `File::create` する
    fn new(output: &Path) -> Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        // プロセス内の他ジョブと衝突しないよう pid + counter を付与
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let pid = std::process::id();
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let parent = output
            .parent()
            .ok_or_else(|| Error::Message("output has no parent directory".into()))?;
        let file_name = output
            .file_name()
            .ok_or_else(|| Error::Message("output has no file name".into()))?
            .to_string_lossy()
            .into_owned();
        let tmp_path = parent.join(format!(".{file_name}.{pid}.{counter}.tmp"));
        // File::create は truncate するため、既存の同名 tmp があれば削除される
        // (pid + counter 衝突時のみ発生、確率は極小)
        std::fs::File::create(&tmp_path)?;
        Ok(Self {
            path: tmp_path,
            committed: false,
        })
    }

    /// 一時ファイルのパスを返す
    fn path(&self) -> &Path {
        &self.path
    }

    /// `rename` 成功後に呼び、`Drop` での削除を防ぐ
    fn commit(mut self) {
        self.committed = true;
    }
}

#[cfg(target_os = "macos")]
impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.committed
            && let Err(e) = std::fs::remove_file(&self.path)
        {
            // 失敗時も処理継続 (ログのみ、業務影響なし)
            tracing::warn!(
                path = %self.path.display(),
                error = %e,
                "failed to remove temp file on drop"
            );
        }
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn write_mp4(
    output: &Path,
    video: Option<VideoOutput>,
    audio: Option<AudioOutput>,
    video_timescale: Option<NonZeroU32>,
) -> Result<()> {
    struct MuxWriter {
        file: std::fs::File,
        position: u64,
    }

    impl MuxWriter {
        /// サンプルデータを書き込み、`data_offset` と `data_size` を返す
        fn write_data(&mut self, data: &[u8]) -> Result<(u64, usize)> {
            self.file.write_all(data)?;
            let offset = self.position;
            let size = data.len();
            self.position += size as u64;
            Ok((offset, size))
        }

        fn flush(&mut self) -> Result<()> {
            self.file.flush().map_err(Error::from)
        }
    }

    let mut muxer = Mp4FileMuxer::new().map_err(|e| Error::Message(format!("muxer init: {e}")))?;
    let initial_bytes = muxer.initial_boxes_bytes().to_vec();

    let temp = TempFile::new(output)?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(temp.path())?;
    let mut writer = MuxWriter {
        file,
        position: 0,
    };
    writer.file.write_all(&initial_bytes)?;
    writer.position = initial_bytes.len() as u64;

    // 音声の timescale は出力サンプルレートを使う (timestamp がサンプル単位のため)
    let audio_timescale = audio.as_ref().and_then(|a| NonZeroU32::new(a.sample_rate));

    // 時系列順にマージするためのイベントリスト
    struct MergeEvent {
        timestamp: u64,
        timescale: NonZeroU32,
        kind: OutputKind,
    }

    impl Ord for MergeEvent {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            // 有理数比較: self.timestamp / self.timescale vs other.timestamp / other.timescale
            // クロス乗算で整数演算のみで比較する
            let a = self.timestamp as u128 * other.timescale.get() as u128;
            let b = other.timestamp as u128 * self.timescale.get() as u128;
            a.cmp(&b)
        }
    }

    impl PartialOrd for MergeEvent {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for MergeEvent {
        fn eq(&self, other: &Self) -> bool {
            self.cmp(other) == std::cmp::Ordering::Equal
        }
    }

    impl Eq for MergeEvent {}

    let mut events: Vec<MergeEvent> = Vec::new();

    if let Some(v) = &video {
        let ts = video_timescale.expect("video timescale");
        for s in &v.samples {
            events.push(MergeEvent {
                timestamp: s.timestamp,
                timescale: ts,
                kind: OutputKind::Video(EncodedVideoSample {
                    data: s.data.clone(),
                    keyframe: s.keyframe,
                    timestamp: s.timestamp,
                    duration: s.duration,
                    composition_time_offset: s.composition_time_offset,
                }),
            });
        }
    }
    if let Some(a) = &audio {
        let ts = audio_timescale.expect("audio timescale");
        for s in &a.samples {
            events.push(MergeEvent {
                timestamp: s.timestamp,
                timescale: ts,
                kind: OutputKind::Audio(EncodedAudioSample {
                    data: s.data.clone(),
                    timestamp: s.timestamp,
                    duration: s.duration,
                    composition_time_offset: s.composition_time_offset,
                }),
            });
        }
    }
    events.sort();

    let video_entry = video.as_ref().map(|v| v.sample_entry.clone());
    let audio_entry = audio.as_ref().map(|a| a.sample_entry.clone());
    let video_ts = video_timescale;
    let audio_ts = audio_timescale;

    let mut video_entry_sent = false;
    let mut audio_entry_sent = false;

    for event in events {
        match event.kind {
            OutputKind::Video(s) => {
                let (data_offset, data_size) = writer.write_data(&s.data)?;
                let ts = video_ts.expect("video timescale");
                let sample = MuxSample {
                    track_kind: TrackKind::Video,
                    sample_entry: if video_entry_sent {
                        None
                    } else {
                        video_entry_sent = true;
                        video_entry.clone()
                    },
                    keyframe: s.keyframe,
                    timescale: ts,
                    duration: s.duration,
                    composition_time_offset: s.composition_time_offset,
                    data_offset,
                    data_size,
                };
                muxer
                    .append_sample(&sample)
                    .map_err(|e| Error::Message(format!("mux append video: {e}")))?;
            }
            OutputKind::Audio(s) => {
                let (data_offset, data_size) = writer.write_data(&s.data)?;
                let ts = audio_ts.expect("audio timescale");
                let sample = MuxSample {
                    track_kind: TrackKind::Audio,
                    sample_entry: if audio_entry_sent {
                        None
                    } else {
                        audio_entry_sent = true;
                        audio_entry.clone()
                    },
                    keyframe: false,
                    timescale: ts,
                    duration: s.duration,
                    composition_time_offset: s.composition_time_offset,
                    data_offset,
                    data_size,
                };
                muxer
                    .append_sample(&sample)
                    .map_err(|e| Error::Message(format!("mux append audio: {e}")))?;
            }
        }
    }

    // muxer をファイナライズする
    let finalized = muxer
        .finalize()
        .map_err(|e| Error::Message(format!("mux finalize: {e}")))?;
    for (offset, bytes) in finalized.offset_and_bytes_pairs() {
        writer.file.seek(SeekFrom::Start(offset))?;
        writer.file.write_all(bytes)?;
    }
    writer.flush()?;

    // atomic rename で出力先に置換。失敗時は Drop で tmp 削除
    std::fs::rename(temp.path(), output)?;
    temp.commit();
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn write_mp4(
    _output: &Path,
    _video: Option<VideoOutput>,
    _audio: Option<AudioOutput>,
    _video_timescale: Option<NonZeroU32>,
) -> Result<()> {
    Err(Error::Message(
        "mp4dropXPd は macOS 以外ではエンコードできません".into(),
    ))
}
