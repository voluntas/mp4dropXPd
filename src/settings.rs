/// 永続化したいアプリ設定（将来の .toml 保存を見越した形）
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub overwrite: bool,
    pub auto_encode_on_drop: bool,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            overwrite: false,
            auto_encode_on_drop: true,
            // 0 = 未指定 (エンコード時に入力の実ビットレートを引き継ぐ)
            video_bitrate_kbps: 0,
            audio_bitrate_kbps: 0,
        }
    }
}
