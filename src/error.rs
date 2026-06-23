use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    /// 汎用フォールバック (仕様由来の判断による文字列に限定)
    Message(String),
    UnsupportedFormat {
        path: PathBuf,
    },
    /// 破損入力などでサンプルデータ範囲がファイルサイズを超過
    InvalidSampleRange {
        start: usize,
        end: usize,
        file_size: usize,
    },
    /// drain_vt_encoder のストールタイムアウト
    DrainTimeout {
        elapsed: String,
        expected: usize,
        received: usize,
    },
    /// 映像/音声トラックがどちらも存在しない
    NoTrack,
    /// 個別トラックのエンコード失敗 (部分成功時に使用)
    TrackEncode {
        track: &'static str,
        message: String,
    },
    /// MP4 demux エラー
    Demux(shiguredo_mp4::demux::DemuxError),
    #[cfg(target_os = "macos")]
    VideoToolbox(shiguredo_video_toolbox::Error),
    #[cfg(target_os = "macos")]
    AudioToolbox(shiguredo_audio_toolbox::Error),
    Opus(shiguredo_opus::Error),
    SvtAv1(shiguredo_svt_av1::Error),
    Mp4(shiguredo_mp4::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Message(s) => f.write_str(s),
            Self::UnsupportedFormat { path } => {
                write!(f, "unsupported input: {}", path.display())
            }
            Self::InvalidSampleRange {
                start,
                end,
                file_size,
            } => {
                write!(f, "sample data out of range: {start}..{end}, file size {file_size}")
            }
            Self::DrainTimeout {
                elapsed,
                expected,
                received,
            } => {
                write!(
                    f,
                    "video encoder stalled: elapsed {elapsed}, expected {expected} frames, received {received}"
                )
            }
            Self::NoTrack => write!(f, "no video/audio track in input"),
            Self::TrackEncode { track, message } => write!(f, "{track} encode failed: {message}"),
            Self::Demux(e) => write!(f, "mp4 demux error: {e}"),
            #[cfg(target_os = "macos")]
            Self::VideoToolbox(e) => write!(f, "video toolbox: {e:?}"),
            #[cfg(target_os = "macos")]
            Self::AudioToolbox(e) => write!(f, "audio toolbox: {e:?}"),
            Self::Opus(e) => write!(f, "opus: {e:?}"),
            Self::SvtAv1(e) => write!(f, "svt-av1: {e:?}"),
            Self::Mp4(e) => write!(f, "mp4: {e:?}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(target_os = "macos")]
impl From<shiguredo_video_toolbox::Error> for Error {
    fn from(e: shiguredo_video_toolbox::Error) -> Self {
        Self::VideoToolbox(e)
    }
}

#[cfg(target_os = "macos")]
impl From<shiguredo_audio_toolbox::Error> for Error {
    fn from(e: shiguredo_audio_toolbox::Error) -> Self {
        Self::AudioToolbox(e)
    }
}

impl From<shiguredo_opus::Error> for Error {
    fn from(e: shiguredo_opus::Error) -> Self {
        Self::Opus(e)
    }
}

impl From<shiguredo_svt_av1::Error> for Error {
    fn from(e: shiguredo_svt_av1::Error) -> Self {
        Self::SvtAv1(e)
    }
}

impl From<shiguredo_mp4::Error> for Error {
    fn from(e: shiguredo_mp4::Error) -> Self {
        Self::Mp4(e)
    }
}

impl From<shiguredo_mp4::demux::DemuxError> for Error {
    fn from(e: shiguredo_mp4::demux::DemuxError) -> Self {
        Self::Demux(e)
    }
}
