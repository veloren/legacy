// Vorbis decoder written in Rust
//
// This example file is licensed
// under the CC-0 license:
// https://creativecommons.org/publicdomain/zero/1.0/

extern crate alto;
extern crate byteorder;
extern crate lewton;

use alto::Context;
use audio::openal::BufferInternal;
use lewton::{inside_ogg::OggStreamReader, VorbisError};
use std::{fs::File, path::Path, time::Duration};

pub(crate) fn load_ogg(file_path: &Path, ctx: &Context) -> Result<BufferInternal, VorbisError> {
    debug!("Opening file: {:?}", file_path);
    let f = File::open(file_path).expect("Can't open file");

    // Prepare the reading
    let mut srr = try!(OggStreamReader::new(f));

    // Prepare the playback.
    let sample_rate = srr.ident_hdr.audio_sample_rate as i32;

    if srr.ident_hdr.audio_channels > 2 {
        // the openal crate can't process these many channels directly
        error!("Stream error: {} channels are too many!", srr.ident_hdr.audio_channels);
    }

    debug!("Sample rate: {}", srr.ident_hdr.audio_sample_rate);

    // Start Reading
    let mut data = Vec::new();
    let mut len_play = 0.0;
    let sample_channels = srr.ident_hdr.audio_channels as f32 * srr.ident_hdr.audio_sample_rate as f32;
    while let Some(pck_samples) = try!(srr.read_dec_packet_itl()) {
        //println!("Decoded packet no {}, with {} samples.", n, pck_samples.len());
        len_play += pck_samples.len() as f32 / sample_channels;
        data.push(pck_samples);
    }
    let duration = Duration::from_millis((len_play * 1000.0) as u64);
    debug!("The file {:?} is {} s long.", file_path, len_play);

    Ok(BufferInternal {
        data,
        duration,
        sample_rate,
        audio_channels: srr.ident_hdr.audio_channels,
    })
}
