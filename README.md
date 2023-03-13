### Audio Resampler

This is a Rust program that resamples audio files (mono or stereo) to a fixed sample rate of 44100 Hz stereo. The resampling is performed using the rubato crate which provides high-quality resampling using a variety of resampling algorithms.

## Usage

To use this program, you will need to have Rust installed on your machine.

Clone the repository:

git clone https://github.com/Khubaib96/rustAudioMixing.git

1. Navigate to the cloned directory: cd audio-resampler
2. Build the project: cargo build --release
3. Run the program: ./target/release/audio-resampler <input_file_path> <output_file_path>
4. Replace <input_file_path> with the path to the audio file you want to resample, and <output_file_path> with the path where you want to save the resampled audio file.

# For example:
### ./target/release/audio-resampler input.wav output.wav

# Dependencies
This program uses the following crates:

1. **hound** for reading and writing WAV files
2. **rubato** for resampling audio
3. **std** and **collections** from the Rust standard library