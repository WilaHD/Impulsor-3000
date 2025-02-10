use crate::impuls::ImpulsModel;

pub mod audio;
use audio::AudioModel;

pub enum ImpulsFileType {
    Audio(AudioModel),
    Pdf(ImpulsModel),
    Unknown(String),
}