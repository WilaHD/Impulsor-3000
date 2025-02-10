use std::{ffi::OsStr, fs::File, io::{BufWriter, Write}, path::PathBuf};

use lame::Lame;
use symphonia::{core::{audio::{AudioBufferRef, Signal}, codecs::DecoderOptions, io::MediaSourceStream, meta::MetadataOptions}, default::{get_codecs, get_probe}};


pub const SUPPORTED_AUDIO_TYPES: [&str; 3] = ["m4a", "ogg", "mp4"];


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioConvertingState {
    Default,
    Success,
    Failure(String),
}

pub struct AudioModel {
    pub state: AudioConvertingState,
    path_input: PathBuf,
    path_output: PathBuf,
}

impl AudioModel {
    pub fn build(path_input: PathBuf) -> Self {
        let mut path_output = path_input.clone();
        path_output.set_extension("mp3");

        AudioModel {
            state: AudioConvertingState::Default,
            path_input,
            path_output,
        }
    }

    pub fn get_file_name(&self) -> String {
        return self.path_input.file_name().unwrap_or(OsStr::new("unknown")).to_string_lossy().to_string();
    }

    pub fn get_path_input_str(&self) -> String {
        return self.path_input.to_string_lossy().to_string()
    }

    pub fn get_path_output_str(&self) -> String {
        return self.path_output.to_string_lossy().to_string()
    }

    pub fn convert(&mut self) {
        match convert(&self.path_input, &self.path_output) {
            Ok(_) => self.state = AudioConvertingState::Success,
            Err(e) => {
                self.state = AudioConvertingState::Failure(e.to_string())
            },
        }
    }
}


fn convert(input_file: &PathBuf, output_file:&PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Add the directory containing the LAME library to the library path
    #[cfg(target_os = "linux")]
    std::env::set_var("LD_LIBRARY_PATH", "./libs/lame/linux-x64");

    #[cfg(target_os = "macos")]
    std::env::set_var("DYLD_LIBRARY_PATH", "./libs/lame/...");

    #[cfg(target_os = "windows")]
    std::env::set_var("PATH", format!("{};{}", "./libs/lame/win-x64", std::env::var("PATH").unwrap()));

    // Open the input file
    let file = File::open(input_file)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Probe the format of the input file
    let probe = get_probe();
    let mut format = probe
        .format(&Default::default(), mss, &Default::default(), &MetadataOptions::default())?
        .format;

    // Find the default audio track
    let track = format
        .default_track()
        .ok_or("No audio track found in the input file")?;
    let track_id = track.id;
    let codec_params = track.codec_params.clone();
    // Create a decoder for the audio track
    let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    // Prepare the MP3 encoder using LAME
    let mut mp3_output = BufWriter::new(File::create(output_file)?);
    let mut lame_encoder = Lame::new().expect("Failed to initialize LAME");
    lame_encoder
        .set_sample_rate(codec_params.sample_rate.ok_or("Missing sample rate")?)
        .expect("Failed to set sample rate");
    lame_encoder.init_params().expect("Failed to initialize LAME encoder");

    // Process packets and convert to MP3
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() == track_id {
            let decoded = decoder.decode(&packet)?;

            // Handle decoded audio frame
            if let AudioBufferRef::F32(buffer) = decoded {
                let num_channels = buffer.spec().channels.count();
                let mut left_channel = Vec::new();
                let mut right_channel = Vec::new();

                // Separate left and right channels
                for frame_idx in 0..buffer.frames() {
                    for channel_idx in 0..num_channels {
                        let sample = buffer.chan(channel_idx)[frame_idx];
                        let sample_i16 = (sample * i16::MAX as f32) as i16;
                        if channel_idx == 0 {
                            left_channel.push(sample_i16);
                        } else if channel_idx == 1 {
                            right_channel.push(sample_i16);
                        }
                    }
                }

                // Encode the PCM data into MP3
                let mut mp3_buffer = vec![0; 4096];
                let encoded_bytes = lame_encoder
                    .encode(&left_channel, &right_channel, &mut mp3_buffer)
                    .expect("LAME encoding failed");
                mp3_output.write_all(&mp3_buffer[..encoded_bytes])?;
            }
        }
    }

    // Finalize MP3 encoding
    let mut mp3_buffer = vec![0; 4096];
    let encoded_bytes = lame_encoder
        .encode(&[], &[], &mut mp3_buffer)
        .expect("LAME finalization failed");
    mp3_output.write_all(&mp3_buffer[..encoded_bytes])?;

    println!("Conversion completed successfully: {} -> {}", input_file.to_string_lossy(), output_file.to_string_lossy(),);
    Ok(())
}
