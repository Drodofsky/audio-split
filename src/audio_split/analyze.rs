use std::{path::PathBuf, time::Duration};

use tokio::process::Command;

use crate::audio_split::error::Error;

pub async fn detect_silence(
    path: impl Into<PathBuf> + Send + 'static,
    threshold_db: f32,
    min_silence_duration: Duration,
) -> Result<Vec<Duration>, Error> {
    let path: PathBuf = path.into();
    let mut time_stamps = Vec::new();

    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-nostats")
        .arg("-i")
        .arg(path)
        .arg("-af")
        .arg(format!(
            "silencedetect=n={}dB:d={}",
            threshold_db,
            min_silence_duration.as_secs_f32()
        ))
        .arg("-f")
        .arg("null")
        .arg("-")
        .output()
        .await
        .unwrap();
    if !output.status.success() {
        panic!("{output:?}")
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut start_time: Option<f64> = None;
    for line in stderr.lines() {
        if !line.contains("silencedetect") {
            continue;
        }
        if let Some(t) = parse_secs_from_line(line, "silence_start:") {
            start_time = Some(t);
        } else if let Some(t) = parse_secs_from_line(line, "silence_end:") {
            if let Some(start) = start_time {
                let time = (start + t) / 2.;
                time_stamps.push(Duration::from_secs_f64(time));
            }
        }
    }
    println!("{time_stamps:#?}");

    Ok(time_stamps)
}

fn parse_secs_from_line(line: &str, key: &str) -> Option<f64> {
    let idx = line.find(key)?;
    let rest = &line[idx + key.len()..];
    let num = rest.split('|').next()?.trim();
    num.parse::<f64>().ok()
}
