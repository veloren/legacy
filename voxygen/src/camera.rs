use std::f32::consts::PI;

use vek::{Mat4, Vec2, Vec3, Vec4};

pub struct Camera {
    focus: Vec3<f32>,
    ori: Vec2<f32>,
    aspect_ratio: f32,
    fov: f32,
    zoom: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            focus: Vec3::zero(),
            ori: Vec2::zero(),
            aspect_ratio: 1.618,
            fov: 1.7,
            zoom: 10.0,
        }
    }

    pub fn get_mats(&self) -> (Mat4<f32>, Mat4<f32>) {
        let mut view = Mat4::identity();

        view *= Mat4::<f32>::translation_3d(Vec3::new(0.0, 0.0, -self.zoom))
            * Mat4::rotation_x(self.ori.y)
            * Mat4::rotation_y(self.ori.x);

        // Apply anti-OpenGL correction
        view *= Mat4::rotation_3d(PI / 2.0, -Vec4::unit_x());

        view *= Mat4::<f32>::translation_3d(-self.focus);

        let perspective = Mat4::<f32>::perspective_rh_no(self.fov, self.aspect_ratio, 0.1, 10000.0);

        (view, perspective)
    }

    pub fn rotate_by(&mut self, dangle: Vec2<f32>) {
        self.ori += dangle;
        if self.ori.y < -PI / 2.0 {
            self.ori.y = -PI / 2.0;
        } else if self.ori.y > PI / 2.0 {
            self.ori.y = PI / 2.0;
        }
    }

    pub fn zoom_by(&mut self, delta: f32) {
        self.zoom += delta;
        if self.zoom < 0.0 {
            self.zoom = 0.0;
        }
    }

    pub fn get_pos(&self, mats: Option<&(Mat4<f32>, Mat4<f32>)>) -> Vec3<f32> {
        // TODO: We should cache result or find a better way of computing it to avoid
        // computing the matrix inverse (expensive to compute) every time we want to
        // get the camera position.
        let p = match mats {
            Some(m) => m.0.inverted() * Vec4::new(0.0, 0.0, 0.0, 1.0),
            None => self.get_mats().0.inverted() * Vec4::new(0.0, 0.0, 0.0, 1.0),
        };

        Vec3::new(p.x, p.y, p.z)
    }

    #[allow(dead_code)]
    pub fn ori(&self) -> &Vec2<f32> { &self.ori }

    #[allow(dead_code)]
    pub fn set_aspect_ratio(&mut self, ratio: f32) { self.aspect_ratio = ratio; }
    #[allow(dead_code)]
    pub fn set_fov(&mut self, fov: f32) { self.fov = fov; }
    #[allow(dead_code)]
    pub fn set_focus(&mut self, focus: Vec3<f32>) { self.focus = focus; }
    #[allow(dead_code)]
    pub fn get_zoom(&mut self) -> f32 { self.zoom }
    #[allow(dead_code)]
    pub fn set_zoom(&mut self, zoom: f32) { self.zoom = zoom; }
}
