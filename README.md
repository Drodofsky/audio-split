# audio-split

audio-split is a minimal **audio segmentation** tool written in Rust using iced for the GUI and ffmpeg for audio processing.
It is designed to automatically **detect silence** in audio files and split them into smaller segments ("spans") that can be reviewed, named, and exported.
The project is in early development; bugs and incomplete error handling are to be expected. This program requires *ffmpeg* to be installed and available in the system PATH.



![](media/audio_split.gif)



### Installation (Linux)

```bash
git clone https://github.com/Drodofsky/audio-split
cd audio-split
cargo install --path .
mkdir -p ~/.local/share/applications
cp audio-split.desktop ~/.local/share/applications/
mkdir -p ~/.local/share/icons/hicolor/256x256/apps
cp media/icon.png ~/.local/share/icons/hicolor/256x256/apps/audio-split.png
update-desktop-database ~/.local/share/applications/ 
```

