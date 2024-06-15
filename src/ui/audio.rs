use std::{fs::File, path::Path};

use rodio::{
    source::{Buffered, Source},
    Decoder,
};

pub struct SoundEffects {
    pub start: Buffered<Decoder<File>>,
    // pub end: Buffered<Decoder<File>>,
    pub perfect: Buffered<Decoder<File>>,
    pub valid: Buffered<Decoder<File>>,
    pub firework: Buffered<Decoder<File>>,
}

fn buffer_sound_effect<P: AsRef<Path>>(path: P) -> Buffered<Decoder<File>> {
    let sound_file = File::open(&path)
        .unwrap_or_else(|_| panic!("Should be able to load `{}`", path.as_ref().display()));
    let source = Decoder::new(sound_file).unwrap_or_else(|_| {
        panic!(
            "Should be able to decode audio file `{}`",
            path.as_ref().display()
        )
    });
    source.buffered()
}

impl Default for SoundEffects {
    fn default() -> Self {
        SoundEffects {
            start: buffer_sound_effect("./assets/start.mp3"),
            //end: buffer_sound_effect("./assets/end.mp3"),
            perfect: buffer_sound_effect("./assets/perfect.mp3"),
            valid: buffer_sound_effect("./assets/valid.mp3"),
            firework: buffer_sound_effect("./assets/firework.mp3"),
        }
    }
}
