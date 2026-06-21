#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioCodec {
    Aac,
    #[default]
    Opus,
}

impl AudioCodec {
    pub const ALL: [Self; 2] = [Self::Aac, Self::Opus];

    pub fn label(self) -> &'static str {
        match self {
            Self::Aac => "AAC",
            Self::Opus => "Opus",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoCodec {
    H264,
    H265,
    #[default]
    Av1,
}

impl VideoCodec {
    pub const ALL: [Self; 3] = [Self::H264, Self::H265, Self::Av1];

    pub fn label(self) -> &'static str {
        match self {
            Self::H264 => "H.264",
            Self::H265 => "H.265",
            Self::Av1 => "AV1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EncodeRecipe {
    pub audio: AudioCodec,
    pub video: VideoCodec,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
}

impl EncodeRecipe {
    pub fn summary(self) -> String {
        format!("{} + {}", self.audio.label(), self.video.label())
    }

    /// 入力の実ビットレートが分かった場合、UI で未設定 (0) の項目を上書きする
    ///
    /// UI 側でユーザーが明示的にビットレートを指定した場合はそれを優先し、
    /// デフォルト値 (0 = 未指定) のときだけ入力の実ビットレートを使う。
    pub fn with_input_bitrates(
        self,
        video_input_kbps: Option<u32>,
        audio_input_kbps: Option<u32>,
    ) -> Self {
        Self {
            video_bitrate_kbps: if self.video_bitrate_kbps == 0 {
                video_input_kbps.unwrap_or(self.video_bitrate_kbps)
            } else {
                self.video_bitrate_kbps
            },
            audio_bitrate_kbps: if self.audio_bitrate_kbps == 0 {
                audio_input_kbps.unwrap_or(self.audio_bitrate_kbps)
            } else {
                self.audio_bitrate_kbps
            },
            ..self
        }
    }
}
