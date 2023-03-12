extern crate hound;

use std::f32::consts::PI;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::collections::HashMap;

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

fn main() {
    // add file path here
    let files = vec!["src/test.wav","src/test3.wav", "src/test2.wav"];

    //create hashmap to store samples with file name as keys
    let mut filesData: HashMap<String, Vec<i16>> = HashMap::new();
    let mut filesLength: HashMap<String, i32> = HashMap::new();


    //loop over files to store in hashmap (here we assume all formats of wavfiles are same except length of data)
    for file in files {
        filesData.insert((file.split('/').last().unwrap().split('.').next().unwrap()).to_string(), openWaveFile(file));
        filesLength.insert((file.split('/').last().unwrap().split('.').next().unwrap()).to_string(), filesData.get(&((file.split('/').last().unwrap().split('.').next().unwrap()).to_string())).unwrap().len() as i32);
        println!("{:?}", filesLength.get(&((file.split('/').last().unwrap().split('.').next().unwrap()).to_string())).unwrap());
    }

    //loop over files and get longest length and file name. Then add zeros to the end of the other short files and replace the samples in the hashmap
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

    // creating a output vector that adds all the samples together and divide by the number of files
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

    //write samples to new file
    writeWavFile("src/out.wav", samples, 48000, 16, 1);
}
