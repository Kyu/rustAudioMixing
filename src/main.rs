extern crate hound;
extern crate rubato;

use hound::{SampleFormat, WavSpec, WavWriter};
use std::collections::HashMap;
use std::time::Instant;
use rubato::{Resampler, SincFixedIn, InterpolationType, InterpolationParameters, WindowFunction};
use rayon::prelude::*;

/**
 * This function opens a WAV file and returns a vector of samples
 * @param file_path - the path to the WAV file
 * @return a vector of samples
 */
fn open_wave_file(file_path: &str) -> Vec<f64> {
    // open the WAV file
    let file = match hound::WavReader::open(file_path) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open WAV file: {}", e),
    };

    // get the WAV file spec
    let spec = file.spec();
    let num_channels = spec.channels as usize;
    let sample_rate = spec.sample_rate;
    let bit_depth = spec.bits_per_sample;
    let max_sample_value = (2.0_f64.powi(bit_depth as i32 - 1) - 1.0) as i32;
    let num_samples = file.duration() as usize;

    // If the audio is mono, treat it as stereo by duplicating the single channel
    let mut channel_data: Vec<Vec<f64>> = vec![Vec::new(); if spec.channels == 1 { 2 } else { spec.channels as usize }];
    if num_channels == 1 {
        let mono_samples = file.into_samples::<i32>();
        // normalize the samples in the mono channel in the range [-1.0, 1.0]
        for sample in mono_samples {
            let value = match sample {
                Ok(v) => v,
                Err(e) => panic!("Failed to read sample: {}", e),
            };
            channel_data[0].push(value as f64 / max_sample_value as f64);
            channel_data[1].push(value as f64 / max_sample_value as f64);
        }
    } else {
        // Read the sample data as an iterator of interleaved samples
        let samples = file.into_samples::<i32>();


        // Iterate over each interleaved sample in the sample iterator
        for (i, sample) in samples.enumerate() {
            // Determine the channel index based on whether the sample index is odd or even
            let channel = if i % 2 == 0 { 0 } else { 1 };
            // Extract the sample value and push it into the corresponding channel vector in the channel data vector
            let value = match sample {
                Ok(v) => v,
                Err(e) => panic!("Failed to read sample: {}", e),
            };
            channel_data[channel].push(value as f64 / max_sample_value as f64);
        }
    }

    // if sample rate is not 44100, resample the audio
    let mut channel_resampled_data = channel_data.clone();
    if sample_rate != 44100 {
        // parameters for the resampler
        let params = InterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: InterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        // create a resampler that converts the sample rate to 44100
        let mut resampler = SincFixedIn::<f64>::new(
            44100f64 / sample_rate as f64,
            2.0,
            params,
            num_samples,
            2,
        ).unwrap();
        channel_resampled_data = resampler.process(&channel_data, None).unwrap();
    }

    // Convert the channel data vector into a single vector of interleaved samples
    let mut resampled_samples_f64 = Vec::new();
    for i in 0..channel_resampled_data[0].len() {
        for channel in 0..if spec.channels == 1 { 2 } else { spec.channels as usize } {
            resampled_samples_f64.push(channel_resampled_data[channel][i]);
        }
    }
    resampled_samples_f64
}

/**
 * This function writes a vector of samples to a WAV file
 * @param file_path - the path to the WAV file
 * @param samples - a vector of samples
 * @param sample_rate - the sample rate of the WAV file
 * @param bit_depth - the bit depth of the WAV file
 * @param num_channels - the number of channels of the WAV file
 */
fn write_wav_file(file_path: &str, samples: Vec<i32>, sample_rate: u32, bit_depth: u16, num_channels: u16) {
    // create a new WAV file
    let mut writer = WavWriter::create(file_path, WavSpec {
        channels: num_channels,
        sample_rate,
        bits_per_sample: bit_depth,
        sample_format: SampleFormat::Int,
    }).unwrap();

    // write the samples to the new WAV file
    for sample in samples {
        writer.write_sample(sample).unwrap();
    }
}

/**
 * This function takes a vector of WAV files and combines them into a single WAV file
 * @param files - a vector of WAV file paths
 * @return a vector of samples
 */
fn process_files(files: Vec<&str>) -> Vec<i32> {
    let start_time = Instant::now();

    // Read each file and store its data in a HashMap along with its length in a separate HashMap in multiple threads
    let mut files_data: HashMap<String, Vec<f64>> = files.par_iter()
        .map(|file| {
            let filename = file.split('/').last().unwrap().split('.').next().unwrap().to_owned();
            let data = open_wave_file(file);
            (filename, data)
        })
        .collect();

    let files_length: HashMap<String, i32> = files_data.par_iter()
        .map(|(filename, data)| (filename.clone(), data.len() as i32))
        .collect();

    // Determine the longest file and pad the shorter files with zeros
    let longest_file = files_length.iter().max_by_key(|(_, &length)| length).unwrap().0;
    let longest_length = files_length.get(longest_file).unwrap();
    for (key, value) in &mut files_data {
        if key != longest_file {
            let padding = vec![0.0; (longest_length - value.len() as i32) as usize];
            value.extend_from_slice(&padding);
        }
    }

    // Combine the samples from each file and calculate the average and normalize the loudness
    let mut samples = files_data.remove(longest_file).unwrap();
    for (_, value) in &files_data {
        for (s1, s2) in samples.iter_mut().zip(value.iter()) {
            *s1 += *s2;
        }
    }
    // multiply by max sample value to normalize loudness
    for s in &mut samples {
        *s = (*s * 8388607.0).max(-8388580.0).min(8388580.0);
    }

    let end_time = Instant::now();
    println!("time taken in processing: {:?}", end_time.duration_since(start_time));
    samples.iter().map(|&s| s as i32).collect()
}


fn main() {
    // add file path here
    let files = vec!["samples/organ_EM_120.wav", "samples/test.wav", "samples/perc_AM_120.wav", "samples/piano_AM_120.wav", "samples/riser_AM_120.wav", "samples/shakers_AM_120.wav", "samples/synth_AM_120.wav"];

    //write samples to new file
    write_wav_file("out.wav", process_files(files), 44100, 24, 2);
}
