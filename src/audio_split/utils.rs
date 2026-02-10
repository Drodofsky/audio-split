use std::{fs::File, path::PathBuf, time::Duration};

use crate::audio_split::{audio::Audio, audio_span::AudioSpan, error::Error};
use rfd::AsyncFileDialog;
use rodio::Source;
use tokio::process::Command;

pub async fn open_audio_file_dialog(starting_path: Option<PathBuf>) -> Option<String> {
    let mut dialog = AsyncFileDialog::new().set_title("Open Audio File");
    if let Some(path) = starting_path {
        dialog = dialog.set_directory(path);
    }
    dialog
        .pick_file()
        .await
        .and_then(|h| h.path().to_str().map(|s| s.to_string()))
}
pub async fn open_export_folder_dialog(starting_path: Option<PathBuf>) -> Option<String> {
    let mut dialog = AsyncFileDialog::new()
        .set_title("Export Audio Files To Folder")
        .set_can_create_directories(true);
    if let Some(path) = starting_path {
        dialog = dialog.set_directory(path);
    }
    dialog
        .pick_folder()
        .await
        .and_then(|h| h.path().to_str().map(|s| s.to_string()))
}

pub async fn save_audio_files(
    source: PathBuf,
    export_path: PathBuf,
    path_extension: PathBuf,
    spans: Vec<AudioSpan>,
) -> Result<(), Error> {
    for span in spans {
        let mut export_path = export_path.join(span.name());
        export_path.add_extension(&path_extension);
        let base = export_path.parent().unwrap();
        tokio::fs::create_dir_all(base).await.unwrap();
        let output = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-nostats")
            .arg("-y")
            .arg("-i")
            .arg(&source)
            .arg("-ss")
            .arg(fmt_duration(span.start()))
            .arg("-to")
            .arg(fmt_duration(span.end()))
            .arg(&export_path)
            .output()
            .await
            .unwrap();
        assert!(output.status.success());
    }
    Ok(())
}

fn fmt_duration(duration: Duration) -> String {
    format!("{:.6}", duration.as_secs_f32())
}

pub async fn open_audio_file(path: impl Into<PathBuf> + Send + 'static) -> Result<Audio, Error> {
    tokio::task::spawn_blocking(|| {
        let path: PathBuf = path.into();
        let file_name: String = path.file_prefix().unwrap().display().to_string();
        let file = File::open(path).unwrap();
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        let sink = rodio::Sink::connect_new(stream_handle.mixer());
        let source = rodio::Decoder::try_from(file)?;
        let length = source.total_duration().unwrap();
        sink.append(source);
        let span = AudioSpan::new(0, Duration::new(0, 0), length, format!("{file_name}_0"));

        Ok(Audio::new(sink, stream_handle, span, file_name))
    })
    .await
    .unwrap()
}
