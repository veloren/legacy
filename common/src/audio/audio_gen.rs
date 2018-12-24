// Local
use crate::audio::{Buffer, Stream};

pub trait AudioGen {
    fn gen_stream(&self, id: u64, buffer: &Buffer, stream: &Stream);
    fn gen_buffer(&self, id: u64, buffer: &Buffer);
    fn drop_stream(&self, id: u64, buffer: &Buffer, stream: &Stream);
    fn drop_buffer(&self, id: u64, buffer: &Buffer);
}
