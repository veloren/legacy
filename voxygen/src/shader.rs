use std::{fs, io};
use glsl_include;

pub struct Shader {
    data: Vec<u8>,
}

impl Shader {
    pub fn from_file(filename: &str) -> Result<Shader, io::Error> {
        // Utility files
        let noise = fs::read_to_string("shaders/util/noise.glsl")?;

        let shader_code = fs::read_to_string(filename)?;
        let (expanded_code, _) = glsl_include::Context::new()
            .include(&noise, "noise.glsl")
            .expand_to_string(&shader_code).map_err(|e|
                io::Error::new(io::ErrorKind::Other, e)
            )?;

        Ok(Shader { data: expanded_code.into_bytes() })
    }

    pub fn bytes(&self) -> &[u8] { &self.data }
}
