// Local
use audio::{Buffer, Stream};

pub trait AudioGen {
    fn gen_stream(&self, u64, &Buffer, &Stream);
    fn gen_buffer(&self, u64, &Buffer);
    fn drop_stream(&self, u64, &Buffer, &Stream);
    fn drop_buffer(&self, u64, &Buffer);
}
