use nalgebra::{Matrix4, Perspective3, Translation3, Vector2, Vector3};
use std::f32::consts::PI;

pub struct Camera {
    focus: Vector3<f32>,
    ori: Vector2<f32>,
    aspect_ratio: f32,
    fov: f32,
    zoom: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            focus: Vector3::<f32>::new(100.0, 100.0, 50.0),
            ori: Vector2::<f32>::zeros(),
            aspect_ratio: 1.618,
            fov: 1.5,
            zoom: 10.0,
        }
    }

    pub fn get_mats(&self) -> (Matrix4<f32>, Matrix4<f32>) {
        let mut mat = Matrix4::<f32>::identity();

        mat *= Translation3::<f32>::from_vector(Vector3::<f32>::new(0.0, 0.0, -self.zoom)).to_homogeneous();
        mat *= Matrix4::from_scaled_axis(&Vector3::x() * self.ori.y)
            * Matrix4::from_scaled_axis(&Vector3::y() * self.ori.x);

        // Apply anti-OpenGL correction
        mat *= Matrix4::from_scaled_axis(-&Vector3::x() * PI / 2.0);

        mat *= Translation3::<f32>::from_vector(-self.focus).to_homogeneous();

        (
            mat,
            *Perspective3::<f32>::new(self.aspect_ratio, self.fov, 0.1, 10000.0).as_matrix(),
        )
    }

    pub fn rotate_by(&mut self, dangle: Vector2<f32>) {
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

    #[allow(dead_code)]
    pub fn ori(&self) -> &Vector2<f32> { &self.ori }

    #[allow(dead_code)]
    pub fn set_aspect_ratio(&mut self, ratio: f32) { self.aspect_ratio = ratio; }
    #[allow(dead_code)]
    pub fn set_fov(&mut self, fov: f32) { self.fov = fov; }
    #[allow(dead_code)]
    pub fn set_focus(&mut self, focus: Vector3<f32>) { self.focus = focus; }
    #[allow(dead_code)]
    pub fn set_zoom(&mut self, zoom: f32) { self.zoom = zoom; }
}
