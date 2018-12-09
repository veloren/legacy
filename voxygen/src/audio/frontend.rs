use common::{
    audio::{audio_gen::AudioGen, Buffer, Stream},
    util::manager::{Managed, Manager},
};
use parking_lot::{Mutex, RwLock};
use rodio::{Decoder, Device, Source, SpatialSink};
use std::{collections::HashMap, fs::File, io::BufReader, sync::Arc, thread::sleep, time::Duration};
use vek::*;

pub struct AudioFrontend {
    device: Device,
    pos: RwLock<Vec3<f32>>,
    ori: RwLock<Mat4<f32>>,
    streams: RwLock<HashMap<u64, InternalStream>>, //always use SpatialSink even if no possition is used for now
    buffers: RwLock<HashMap<u64, Buffer>>,
}

struct InternalStream {
    pub sink: SpatialSink,
    pub settings: Stream,
}

impl AudioFrontend {
    pub fn new() -> Manager<AudioFrontend> {
        let device = rodio::default_output_device().unwrap();

        Manager::init(AudioFrontend {
            device,
            pos: RwLock::new(Vec3::new(0.0, 0.0, 0.0)),
            ori: RwLock::new(Mat4::identity()),
            streams: RwLock::new(HashMap::new()),
            buffers: RwLock::new(HashMap::new()),
        })
    }

    pub fn set_pos(&self, pos: Vec3<f32>, _vel: Vec3<f32>, ori: Mat4<f32>) {
        *self.pos.write() = pos;
        *self.ori.write() = ori;
        let mut slock = self.streams.write();
        for (id, int) in slock.iter_mut() {
            self.adjust(&int.settings, &mut int.sink);
        }
    }

    fn adjust(&self, stream: &Stream, sink: &mut SpatialSink) {
        const FALLOFF: f32 = 0.13;
        if let Some(pos) = &stream.positional {
            if pos.relative {
                sink.set_emitter_position([pos.pos.x * FALLOFF, pos.pos.y * FALLOFF, pos.pos.z * FALLOFF]);
            } else {
                let lpos = *self.pos.read();
                sink.set_emitter_position([
                    (pos.pos.x - lpos.x) * FALLOFF,
                    (pos.pos.y - lpos.y) * FALLOFF,
                    (pos.pos.z - lpos.z) * FALLOFF,
                ]);
            }
            let lori = *self.ori.read();
            //let mut xyz = lori * Vec4::new(pos.pos.x, pos.pos.y, pos.pos.z , 100.5);
            //TODO: FIXME: Wowowowow, thats some ugly code below to get the relative head direction of the camera working.
            // It works on a flat horizontal plane (which will be enought for 90% of people) but we should have someone with a vector math brain look over it...
            let x = lori.into_row_array();
            let mut xy = Vec3::new(x[0] / 0.813, x[1] / 1.3155, 0.0);
            xy.normalize();
            let mut left_ear = Mat3::rotation_z(3.14) * xy;
            let mut right_ear = xy;
            sink.set_left_ear_position(left_ear.into_array());
            sink.set_right_ear_position(right_ear.into_array());
        }
        sink.set_volume(stream.volume);
    }

    fn create_source(&self, buffer: &Buffer) -> Decoder<BufReader<File>> {
        match buffer {
            Buffer::File(file) => {
                let file = std::fs::File::open(file).unwrap();
                rodio::Decoder::new(BufReader::new(file)).unwrap()
            },
            Buffer::Raw(..) => {
                panic!("raw buffers not implemented yet");
            },
        }
    }
}

impl AudioGen for AudioFrontend {
    fn gen_stream(&self, id: u64, buffer: &Buffer, stream: &Stream) {
        let mut slock = self.streams.write();
        let lock = self.buffers.read();
        if let Some(buffer) = lock.get(&stream.buffer) {
            let src = self.create_source(buffer);
            let mut sink = rodio::SpatialSink::new(&self.device, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]);
            self.adjust(stream, &mut sink);
            sink.append(src);
            let mut internal = InternalStream {
                sink,
                settings: stream.clone(),
            };
            //p.src.play();
            slock.insert(id, internal);
        }
    }

    fn gen_buffer(&self, id: u64, buffer: &Buffer) {
        debug!("generate buffer: {:?}", buffer);
        self.buffers.write().insert(id, buffer.clone());
    }

    fn drop_stream(&self, id: u64, buffer: &Buffer, stream: &Stream) {
        let mut slock = self.streams.write();
        if let Some(p) = slock.get_mut(&id) {
            //p.src.stop();
            slock.remove(&id);
        }
    }

    fn drop_buffer(&self, id: u64, buffer: &Buffer) {}
}

impl Managed for AudioFrontend {
    fn init_workers(&self, manager: &mut Manager<Self>) {
        // Background Sound
        Manager::add_worker(manager, |audio, running, mut mgr| {});
    }

    fn on_drop(&self, _: &mut Manager<Self>) {}
}
