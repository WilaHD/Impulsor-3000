use std::{
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use rusty_mp3::{Error as Mp3Error, Mp3Encoder, Mp3EncoderConfig};
use symphonia::{
    core::{
        audio::SampleBuffer, codecs::DecoderOptions, io::MediaSourceStream, meta::MetadataOptions,
    },
    default::{get_codecs, get_probe},
};

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
        return self
            .path_input
            .file_name()
            .unwrap_or(OsStr::new("unknown"))
            .to_string_lossy()
            .to_string();
    }

    pub fn get_path_input_str(&self) -> String {
        return self.path_input.to_string_lossy().to_string();
    }

    pub fn get_path_output_str(&self) -> String {
        return self.path_output.to_string_lossy().to_string();
    }

    pub fn convert(&mut self) {
        match convert(&self.path_input, &self.path_output) {
            Ok(_) => self.state = AudioConvertingState::Success,
            Err(e) => self.state = AudioConvertingState::Failure(e.to_string()),
        }
    }
}

fn convert(input_file: &PathBuf, output_file: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Open the input file
    let file = File::open(input_file)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Probe the format of the input file
    let probe = get_probe();
    let mut format = probe
        .format(
            &Default::default(),
            mss,
            &Default::default(),
            &MetadataOptions::default(),
        )?
        .format;

    // Find the default audio track
    let track = format
        .default_track()
        .ok_or("No audio track found in the input file")?;
    let track_id = track.id;
    // Create a decoder for the audio track
    let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    let mut encoder = Mp3Encoder::new(Mp3EncoderConfig {
        bitrate_kbps: 128,
        vbr_quality: None,
    });
    let mut input_spec = None;
    let mut has_audio = false;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(error) => return Err(error.into()),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = decoder.decode(&packet)?;
        let spec = *decoded.spec();
        let channels = spec.channels.count();
        if !(1..=2).contains(&channels) {
            return Err(format!("Unsupported channel count: {channels}").into());
        }
        if let Some(previous) = input_spec {
            if previous != spec {
                return Err("Audio format changed during conversion".into());
            }
        } else {
            input_spec = Some(spec);
        }
        let mut samples = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        samples.copy_interleaved_ref(decoded);
        if samples.samples().is_empty() {
            continue;
        }
        if channels == 1 {
            let stereo: Vec<f32> = samples.samples().iter().flat_map(|&s| [s, s]).collect();
            encoder.push_pcm_f32(&stereo, 2, spec.rate)?;
        } else {
            encoder.push_pcm_f32(samples.samples(), 2, spec.rate)?;
        }
        has_audio = true;
    }

    if !has_audio {
        return Err("No audio samples found in the input file".into());
    }
    // Drain only after finish: the first packet then contains the finalized
    // gapless Info header rather than its placeholder.
    encoder.finish();
    let mut mp3_output = BufWriter::new(File::create(output_file)?);
    loop {
        match encoder.next_packet() {
            Ok(packet) => mp3_output.write_all(&packet)?,
            Err(Mp3Error::Eof) => break,
            Err(error) => return Err(error.into()),
        }
    }
    mp3_output.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::Cursor,
        sync::atomic::{AtomicU64, Ordering},
    };
    use symphonia::core::{audio::SampleBuffer, formats::FormatOptions};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "impulsor-audio-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn decode_mp3(path: &PathBuf) -> (u32, usize, Vec<f32>) {
        let file = File::open(path).unwrap();
        let source = MediaSourceStream::new(Box::new(file), Default::default());
        let mut format = get_probe()
            .format(
                &Default::default(),
                source,
                &FormatOptions {
                    enable_gapless: true,
                    ..Default::default()
                },
                &MetadataOptions::default(),
            )
            .unwrap()
            .format;
        let track = format.default_track().unwrap();
        let id = track.id;
        let mut decoder = get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .unwrap();
        let mut pcm = Vec::new();
        let mut rate = 0;
        let mut channels = 0;
        while let Ok(packet) = format.next_packet() {
            if packet.track_id() != id {
                continue;
            }
            let decoded = decoder.decode(&packet).unwrap();
            rate = decoded.spec().rate;
            channels = decoded.spec().channels.count();
            let mut samples = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
            samples.copy_interleaved_ref(decoded);
            pcm.extend_from_slice(samples.samples());
        }
        (rate, channels, pcm)
    }

    #[test]
    fn converts_supported_containers_to_complete_audible_mp3() {
        let dir = TestDir::new();
        for (name, duration) in [("mono.m4a", 0.12), ("stereo.mp4", 1.25), ("mono.ogg", 1.25)] {
            let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/audio")
                .join(name);
            let output = dir.0.join(format!("{name}.mp3"));
            convert(&input, &output).unwrap();
            let (rate, channels, samples) = decode_mp3(&output);
            assert_eq!(channels, 2, "{name}");
            let actual_duration = samples.len() as f32 / channels as f32 / rate as f32;
            assert!(
                (actual_duration - duration).abs() < 0.15,
                "{name}: duration {actual_duration}"
            );
            assert!(samples.iter().any(|s| s.abs() > 0.01), "{name} is silent");
            assert!(
                samples[samples.len() / 2..].iter().any(|s| s.abs() > 0.01),
                "{name} ends in silence"
            );
            if name == "stereo.mp4" {
                assert!(samples
                    .chunks_exact(2)
                    .any(|frame| (frame[0] - frame[1]).abs() > 0.01));
            }
        }
    }

    // WAV exercises Symphonia's integer sample conversion and an unsupported MP3 rate.
    fn wav(rate: u32, samples: u32) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        out.write_all(b"RIFF").unwrap();
        out.write_all(&(36 + samples * 2).to_le_bytes()).unwrap();
        out.write_all(b"WAVEfmt ").unwrap();
        out.write_all(&16u32.to_le_bytes()).unwrap();
        out.write_all(&1u16.to_le_bytes()).unwrap();
        out.write_all(&1u16.to_le_bytes()).unwrap();
        out.write_all(&rate.to_le_bytes()).unwrap();
        out.write_all(&(rate * 2).to_le_bytes()).unwrap();
        out.write_all(&2u16.to_le_bytes()).unwrap();
        out.write_all(&16u16.to_le_bytes()).unwrap();
        out.write_all(b"data").unwrap();
        out.write_all(&(samples * 2).to_le_bytes()).unwrap();
        for i in 0..samples {
            out.write_all(&((i as f32 * 0.1).sin().mul_add(10000.0, 0.0) as i16).to_le_bytes())
                .unwrap();
        }
        out.into_inner()
    }

    #[test]
    fn converts_integer_pcm_and_rejects_unsupported_rate() {
        let dir = TestDir::new();
        let input = dir.0.join("input.wav");
        let output = dir.0.join("output.mp3");
        fs::write(&input, wav(44100, 200)).unwrap();
        convert(&input, &output).unwrap();
        let (_, channels, samples) = decode_mp3(&output);
        assert_eq!(channels, 2);
        assert!(samples.iter().any(|s| s.abs() > 0.01));
        fs::write(&input, wav(12345, 2000)).unwrap();
        assert!(convert(&input, &output).is_err());
    }

    #[test]
    fn invalid_input_and_multichannel_audio_set_failure_state() {
        let dir = TestDir::new();
        let invalid = dir.0.join("invalid.ogg");
        fs::write(&invalid, b"not an audio file").unwrap();
        let mut model = AudioModel::build(invalid);
        model.convert();
        assert!(matches!(model.state, AudioConvertingState::Failure(_)));
        let input = dir.0.join("valid.wav");
        fs::write(&input, wav(44100, 200)).unwrap();
        let mut model = AudioModel::build(input);
        model.path_output = dir.0.join("missing").join("output.mp3");
        model.convert();
        assert!(matches!(model.state, AudioConvertingState::Failure(_)));
        let surround =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/audio/surround.m4a");
        let mut model = AudioModel::build(surround);
        model.path_output = dir.0.join("surround.mp3");
        model.convert();
        assert!(matches!(model.state, AudioConvertingState::Failure(_)));
    }
}
