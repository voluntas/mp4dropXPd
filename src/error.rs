use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Message(String),
    UnsupportedFormat {
        path: PathBuf,
    },
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
        Self::Message(format!("mp4 demux error: {e}"))
    }
}
