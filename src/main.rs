extern crate hound;

use std::f32::consts::PI;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::collections::HashMap;

/**
 * This function opens a WAV file and returns a vector of samples
 * @param filePath - the path to the WAV file
 * @return a vector of samples
 */
fn openWaveFile(filePath: &str) -> Vec<i16> {
    // open the WAV file
    let file = match hound::WavReader::open(filePath) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open WAV file: {}", e),
    };

    // get the WAV file spec
    let spec = file.spec();
    let num_channels = spec.channels;
    let sample_rate = spec.sample_rate;
    let bit_depth = spec.bits_per_sample;

    //read the WAV file
    let samples = file.into_samples::<i16>().collect::<Result<Vec<i16>, _>>().unwrap();

    return samples;
}

/**
 * This function writes a vector of samples to a WAV file
 * @param filePath - the path to the WAV file
 * @param samples - a vector of samples
 * @param sample_rate - the sample rate of the WAV file
 * @param bit_depth - the bit depth of the WAV file
 * @param num_channels - the number of channels of the WAV file
 */
fn writeWavFile(filePath: &str, samples: Vec<i16>, sample_rate: u32, bit_depth: u16, num_channels: u16) {
    // create a new WAV file
    let mut writer = WavWriter::create(filePath, WavSpec {
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
fn process_files(files: Vec<&str>) -> Vec<i16>
{
    //create hashmap to store samples with file name as keys
    let mut filesData: HashMap<String, Vec<i16>> = HashMap::new();
    let mut filesLength: HashMap<String, i32> = HashMap::new();

    // Read each file and store its data in a HashMap along with its length
    for file in files {
        let filename = file.split('/').last().unwrap().split('.').next().unwrap().to_string();
        filesData.insert(filename.clone(), openWaveFile(file));
        filesLength.insert(filename.clone(), filesData.get(&filename).unwrap().len() as i32);
        println!("{:?}", filesLength.get(&filename).unwrap());
    }

    // Determine the longest file and pad the shorter files with zeros
    let mut longestLength = 0;
    let mut longestFile = "".to_string();
    for (key, value) in filesLength.iter() {
        if *value > longestLength {
            longestLength = *value;
            longestFile = key.to_string();
        }
    }
    for (key, value) in filesLength.iter() {
        if *value < longestLength {
            let mut samples = filesData.get(&key.to_string()).unwrap().to_vec();
            let mut i = 0;
            while i < longestLength - *value {
                samples.push(0);
                i += 1;
            }
            filesData.insert(key.to_string(), samples);
        }
    }

    // Combine the samples from each file and calculate the average
    let mut samples = filesData.get(&longestFile).unwrap().to_vec();
    for (key, value) in filesData.iter() {
        if key != &longestFile {
            let mut i = 0;
            while i < value.len() {
                samples[i] = (samples[i] + value[i]) / 2;
                i += 1;
            }
        }
    }
    samples
}


fn main() {
    // add file path here
    let files = vec!["src/test.wav", "src/test3.wav", "src/test2.wav"];

    //write samples to new file
    writeWavFile("src/out.wav", process_files(files), 48000, 16, 1);
}
