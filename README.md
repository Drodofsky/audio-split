# audio-split

audio-split is a small experimental **audio segmentation** tool written in Rust using iced for the GUI and ffmpeg for audio processing.
It is designed to automatically **detect silence** in audio files and split them into smaller segments ("spans") that can be reviewed, namend, and exported into audio files.
The project is in early development; bugs and incomplete error handling are to be expected. This program requires *ffmpeg* to be installed and available in the system PATH.
