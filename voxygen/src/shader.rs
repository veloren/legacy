use glsl_include;
use std::{fs, io};

pub struct Shader {
    data: Vec<u8>,
}

impl Shader {
    pub fn from_file(filename: &str) -> Result<Shader, io::Error> {
        // Utility files
        let common = fs::read_to_string("shaders/util/common.glsl")?;
        let noise = fs::read_to_string("shaders/util/noise.glsl")?;
        let sky = fs::read_to_string("shaders/util/sky.glsl")?;
        let bsdf = fs::read_to_string("shaders/util/bsdf.glsl")?;

        let shader_code = fs::read_to_string(filename)?;
        let (expanded_code, _) = glsl_include::Context::new()
            .include("common.glsl", &common)
            .include("noise.glsl", &noise)
            .include("sky.glsl", &sky)
            .include("bsdf.glsl", &bsdf)
            .expand_to_string(&shader_code)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // println!("{}", &expanded_code);

        Ok(Shader {
            data: expanded_code.into_bytes(),
        })
    }

    pub fn bytes(&self) -> &[u8] { &self.data }
}
