extern crate hound;

use std::f32::consts::PI;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::collections::HashMap;
use std::any::type_name;

/**
 * This function opens a WAV file and returns a vector of samples
 * @param file_path - the path to the WAV file
 * @return a vector of samples
 */
fn openWaveFile(file_path: &str) -> Vec<i32> {
    // open the WAV file
    let file = match hound::WavReader::open(file_path) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open WAV file: {}", e),
    };

    // get the WAV file spec
    let spec = file.spec();
    let num_channels = spec.channels;
    let sample_rate = spec.sample_rate;
    let bit_depth = spec.bits_per_sample;

    //read the WAV file
    let samples = file.into_samples::<i32>().collect::<Result<Vec<i32>, _>>().unwrap();

    return samples;
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
    //create hashmap to store samples with file name as keys
    let mut files_data: HashMap<String, Vec<i32>> = HashMap::new();
    let mut files_length: HashMap<String, i32> = HashMap::new();

    // Read each file and store its data in a HashMap along with its length
    for file in files {
        let filename = file.split('/').last().unwrap().split('.').next().unwrap().to_string();
        files_data.insert(filename.clone(), openWaveFile(file));
        files_length.insert(filename.clone(), files_data.get(&filename).unwrap().len() as i32);
        println!("{:?}", files_length.get(&filename).unwrap());
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
                samples.push(0);
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
                samples[i] = (samples[i] + value[i]);
                // prevent clipping by limiting the sample to 24 bits
                if samples[i] > 8388607 {
                    samples[i] = 8388580;
                } else if samples[i] < -8388608 {
                    samples[i] = -8388580;
                }
                i += 1;
            }
        }
    }
    samples
}


fn main() {
    // add file path here
    let files = vec!["src/organ_EM_120.wav", "src/perc_AM_120.wav", "src/piano_AM_120.wav", "src/riser_AM_120.wav", "src/shakers_AM_120.wav", "src/synth_AM_120.wav"];

    //write samples to new file
    write_wav_file("src/out.wav", process_files(files), 44100, 24, 2);
}
