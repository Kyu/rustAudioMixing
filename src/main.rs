extern crate hound;
extern crate rubato;

use std::f32::consts::PI;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::collections::HashMap;
use std::any::type_name;
use std::time::Instant;
use rubato::{Resampler, SincFixedIn, InterpolationType, InterpolationParameters, WindowFunction};


// Define a function that resamples audio


/**
 * This function opens a WAV file and returns a vector of samples
 * @param file_path - the path to the WAV file
 * @return a vector of samples
 */
fn openWaveFile(file_path: &str) -> Vec<f64> {
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
        44100 as f64 / sample_rate as f64,
        2.0,
        params,
        num_samples,
        2,
    ).unwrap();


    // If the audio is mono, treat it as stereo by duplicating the single channel

    let mut channel_data: Vec<Vec<f64>> = vec![Vec::new(); if spec.channels == 1 { 2 } else { spec.channels as usize }];
    if num_channels == 1 {
        let mut mono_samples = file.into_samples::<i32>();
        // normalize the samples in the mono channel in the range [-1.0, 1.0]
        for sample in mono_samples {
            let value = match sample {
                Ok(v) => v,
                Err(e) => panic!("Failed to read sample: {}", e),
            };
            channel_data[0].push((value as f64 / max_sample_value as f64) as f64);
            channel_data[1].push((value as f64 / max_sample_value as f64) as f64);
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
            channel_data[channel].push((value as f64 / max_sample_value as f64) as f64);
        }
    }

    // if sample rate is not 44100, resample the audio
    let mut channel_resampled_data = channel_data.clone();
    if sample_rate != 44100 {
        channel_resampled_data = resampler.process(&channel_data, None).unwrap();
    }

    // Convert the channel data vector into a single vector of interleaved samples
    let mut resampled_samples_f32 = Vec::new();
    for i in 0..channel_resampled_data[0].len() {
        for channel in 0..if spec.channels == 1 { 2 } else { spec.channels as usize } {
            resampled_samples_f32.push(channel_resampled_data[channel][i]);
        }
    }
    return resampled_samples_f32;
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
    let start_time = Instant::now();
    let mut writer = WavWriter::create(file_path, WavSpec {
        channels: num_channels,
        sample_rate: sample_rate,
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
fn process_files(files: Vec<&str>) -> Vec<i32>
{
    let start_time = Instant::now();
    //create hashmap to store samples with file name as keys
    let mut files_data: HashMap<String, Vec<f64>> = HashMap::new();
    let mut files_length: HashMap<String, i32> = HashMap::new();

    // Read each file and store its data in a HashMap along with its length
    for file in files {
        let filename = file.split('/').last().unwrap().split('.').next().unwrap().to_string();
        files_data.insert(filename.clone(), openWaveFile(file));
        files_length.insert(filename.clone(), files_data.get(&filename).unwrap().len() as i32);
    }

    // Determine the longest file and pad the shorter files with zeros
    let mut longest_length = 0;
    let mut longest_file = "".to_string();
    for (key, value) in files_length.iter() {
        if *value > longest_length {
            longest_length = *value;
            longest_file = key.to_string();
        }
    }
    for (key, value) in files_length.iter() {
        if *value < longest_length {
            let mut samples = files_data.get(&key.to_string()).unwrap().to_vec();
            let mut i = 0;
            while i < longest_length - *value {
                samples.push(0.0);
                i += 1;
            }
            files_data.insert(key.to_string(), samples);
        }
    }

    // Combine the samples from each file and calculate the average and normalize the loudness
    let mut samples = files_data.get(&longest_file).unwrap().to_vec();
    for (key, value) in files_data.iter() {
        if key != &longest_file {
            let mut i = 0;
            while i < value.len() {
                samples[i] = samples[i] + value[i];
                i += 1;
            }
        }
    }
    // multiply by max sample value to normalize loudness
    let mut i = 0;
    while i < samples.len() {
        samples[i] = (samples[i] * 8388607.0) as f64;
        if samples[i] > 8388607.0 {
            samples[i] = 8388580.0;
        }
        if samples[i] < -8388607.0 {
            samples[i] = -8388580.0;
        }
        i += 1;
    }
    let end_time = Instant::now();
    println!(" time taken in processing: {:?}", end_time.duration_since(start_time));
    samples.iter().map(|&s| s as i32).collect()
}

fn main() {
    // add file path here
    let files = vec!["src/organ_EM_120.wav", "src/test.wav", "src/perc_AM_120.wav", "src/piano_AM_120.wav", "src/riser_AM_120.wav", "src/shakers_AM_120.wav", "src/synth_AM_120.wav"];

    //write samples to new file
    write_wav_file("src/out.wav", process_files(files), 44100, 24, 2);
}
