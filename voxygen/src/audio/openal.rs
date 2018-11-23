use alto::{Alto, Context, DistanceModel, Mono, OutputDevice, Source, Stereo, StreamingSource};
use audio::ogg::load_ogg;
use common::{
    audio::{audio_gen::AudioGen, Buffer, Stream},
    util::manager::{Managed, Manager},
};
use parking_lot::{Mutex, RwLock};
use std::{collections::HashMap, thread::sleep, time::Duration};
use vek::*;

pub struct OpenAl {
    alto: Alto,
    device: OutputDevice,
    context: Context,
    streams: RwLock<HashMap<u64, StreamPayload>>,
    buffers: RwLock<HashMap<u64, BufferInternal>>,
}

pub(crate) struct BufferInternal {
    pub data: Vec<Vec<i16>>,
    pub duration: Duration,
    pub sample_rate: i32,
    pub audio_channels: u8,
}

struct StreamPayload {
    src: StreamingSource,
}

impl OpenAl {
    pub fn new() -> Manager<OpenAl> {
        let alto = Alto::load_default().expect("Could not load alto");
        let device = alto.open(None).expect("Could not open device");
        let context = device.new_context(None).expect("Could not create context");

        //context.set_meters_per_unit(10.0);
        context.set_distance_model(DistanceModel::Exponent);

        Manager::init(OpenAl {
            alto,
            device,
            context,
            streams: RwLock::new(HashMap::new()),
            buffers: RwLock::new(HashMap::new()),
        })
    }

    pub fn set_pos(&self, pos: Vec3<f32>, vel: Vec3<f32>, ori: Vec3<f32>) {
        self.context.set_position([pos.x, pos.y, pos.z]).unwrap();
        self.context.set_velocity([vel.x, vel.y, vel.z]).unwrap();
        self.context
            .set_orientation(([ori.x, ori.y, ori.z], [0.0, 0.0, 1.0]))
            .unwrap();
    }

    fn adjust(&self, stream: &Stream, payload: &mut StreamPayload) {
        if let Some(pos) = &stream.positional {
            payload.src.set_relative(pos.relative);
            payload.src.set_position([pos.pos.x, pos.pos.y, pos.pos.z]);
            payload.src.set_velocity([pos.vel.x, pos.vel.y, pos.vel.z]);
        }
        payload.src.set_rolloff_factor(0.7);
        payload.src.set_gain(stream.volume);
    }

    fn create_source(&self, buffer: &BufferInternal) -> StreamingSource {
        let mut str_src = self
            .context
            .new_streaming_source()
            .expect("could not create streaming src");
        let mut n = 0;
        for stream in &buffer.data {
            n += 1;

            let buf = match buffer.audio_channels {
                1 => self.context.new_buffer::<Mono<i16>, _>(&stream, buffer.sample_rate),
                2 => self.context.new_buffer::<Stereo<i16>, _>(&stream, buffer.sample_rate),
                n => panic!("unsupported number of channels: {}", n),
            }
            .unwrap();

            str_src.queue_buffer(buf);
        }
        str_src
    }
}

impl AudioGen for OpenAl {
    fn gen_stream(&self, id: u64, buffer: &Buffer, stream: &Stream) {
        let mut slock = self.streams.write();
        let lock = self.buffers.read();
        if let Some(buffer) = lock.get(&stream.buffer) {
            let src = self.create_source(buffer);
            let mut p = StreamPayload { src };
            self.adjust(stream, &mut p);
            p.src.play();
            slock.insert(id, p);
        }
    }

    fn gen_buffer(&self, id: u64, buffer: &Buffer) {
        match buffer {
            Buffer::File(file) => {
                debug!("generate buffer from: {:?}", file);
                let buf = load_ogg(file, &self.context).unwrap();
                self.buffers.write().insert(id, buf);
            },
            Buffer::Raw(..) => {
                panic!("raw buffers not implemented yet");
            },
        }
    }

    fn drop_stream(&self, id: u64, buffer: &Buffer, stream: &Stream) {
        let mut slock = self.streams.write();
        if let Some(p) = slock.get_mut(&id) {
            p.src.stop();
            slock.remove(&id);
        }
    }

    fn drop_buffer(&self, id: u64, buffer: &Buffer) {}
}

impl Managed for OpenAl {
    fn init_workers(&self, manager: &mut Manager<Self>) {
        // Background Sound
        Manager::add_worker(manager, |audio, running, mut mgr| {});
    }

    fn on_drop(&self, _: &mut Manager<Self>) {}
}
