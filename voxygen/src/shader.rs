use glsl_include;
use std::{env, fs, io};

pub struct Shader {
    data: Vec<u8>,
}

impl Shader {
    pub(crate) fn expand(filename: &str) -> Result<String, io::Error> {
        // Utility files
        let common = fs::read_to_string("shaders/util/common.glsl")?;
        let noise = fs::read_to_string("shaders/util/noise.glsl")?;
        let sky = fs::read_to_string("shaders/util/sky.glsl")?;
        let bsdf = fs::read_to_string("shaders/util/bsdf.glsl")?;
        let luts = fs::read_to_string("shaders/util/luts.glsl")?;

        let shader_code = fs::read_to_string(filename)?;
        let (expanded_code, _) = glsl_include::Context::new()
            .include("common.glsl", &common)
            .include("noise.glsl", &noise)
            .include("sky.glsl", &sky)
            .include("bsdf.glsl", &bsdf)
            .include("luts.glsl", &luts)
            .expand_to_string(&shader_code)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(expanded_code)
    }

    pub fn from_file(filename: &str) -> Result<Shader, io::Error> {
        let expanded_code = Shader::expand(filename)?;

        match env::var("VOXYGEN_DEBUG_SHADERS") {
            Ok(val) => if val.parse::<i32>().unwrap() == 1 {
                println!("{}", &expanded_code);
            },
            _ => {},
        };

        Ok(Shader {
            data: expanded_code.into_bytes(),
        })
    }

    pub fn from_str(code: &str) -> Shader {
        Shader {
            data: code.as_bytes().to_vec(),
        }
    }

    pub fn bytes(&self) -> &[u8] { &self.data }
}
