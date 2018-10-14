use std::f32::consts::PI;

use nalgebra::{Matrix4, Perspective3, Translation3, Vector3, Vector4};
use vek::{Vec2, Vec3, Vec4, Mat4};

use float_cmp::ApproxEqUlps;

fn assert_eq_mat4(m1: &Matrix4<f32>, m2: &Mat4<f32>) {
    for i in 0..4 {
        for j in 0..4 {
            let v1 = unsafe{m1.get_unchecked(i, j)};
            let v2 = m2[(i, j)];
            if !v2.approx_eq_ulps(v1, 10) {
                println!("i: {} j: {} v1: {} v2: {}", i, j, v1, v2);
                println!("{:#?}", m1);
                println!("{:#?}", m2);
                return
            }
        }
    }
}

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
            focus: Vec3::new(100.0, 100.0, 50.0),
            ori: Vec2::zero(),
            aspect_ratio: 1.618,
            fov: 1.5,
            zoom: 10.0,
        }
    }

    pub fn get_mats(&self) -> (Mat4<f32>, Mat4<f32>) {
        let mut view = Mat4::identity();

        view *= Mat4::<f32>::translation_3d(Vec3::new(0.0, 0.0, -self.zoom));
        view *= Mat4::rotation_3d(self.ori.y, Vec4::<f32>::unit_x());
        view *= Mat4::rotation_3d(self.ori.x, Vec4::<f32>::unit_y());

        // Apply anti-OpenGL correction
        view *= Mat4::rotation_3d(PI / 2.0, -Vec4::unit_x());

        view *= Mat4::<f32>::translation_3d(-self.focus);

        let perspective = Mat4::<f32>::perspective_rh_no(self.fov, self.aspect_ratio, 0.1, 10000.0);

        (view, perspective)

    }

    pub fn get_mats_sanity_check(&self) -> (Mat4<f32>, Mat4<f32>) {
        let mut mat = Mat4::identity();
        let mut mat_nalg = Matrix4::identity();

        mat *= Mat4::<f32>::translation_3d(Vec3::new(0.0, 0.0, -self.zoom));
        mat_nalg *= Translation3::from_vector(Vector3::new(0.0, 0.0, -self.zoom)).to_homogeneous();
        assert_eq_mat4(&mat_nalg, &mat);
        
        mat *= Mat4::rotation_3d(self.ori.y, Vec4::<f32>::unit_x());
        mat_nalg *= Matrix4::from_scaled_axis(&Vector3::x() * self.ori.y);
        assert_eq_mat4(&mat_nalg, &mat);

        mat *= Mat4::rotation_3d(self.ori.x, Vec4::<f32>::unit_y());
        mat_nalg *= Matrix4::from_scaled_axis(&Vector3::y() * self.ori.x);
        assert_eq_mat4(&mat_nalg, &mat);

        // Apply anti-OpenGL correction
        mat *= Mat4::rotation_3d(PI / 2.0, -Vec4::unit_x());
        mat_nalg *= Matrix4::from_scaled_axis(-&Vector3::x() * PI / 2.0);
        assert_eq_mat4(&mat_nalg, &mat);

        let trans = Mat4::<f32>::translation_3d(-self.focus);
        let trans_nalg = Translation3::from_vector(-Vector3::new(self.focus.x, self.focus.y, self.focus.z)).to_homogeneous();
        assert_eq_mat4(&trans_nalg, &trans);

        mat *= trans;
        mat_nalg *= trans_nalg; 
        assert_eq_mat4(&mat_nalg, &mat);

        let perspective = Mat4::<f32>::perspective_rh_no(self.fov, self.aspect_ratio, 0.1, 10000.0);
        let persp_nalg = *Perspective3::new(self.aspect_ratio, self.fov, 0.1, 10000.0).as_matrix();
        assert_eq_mat4(&persp_nalg, &perspective);

        (mat, perspective)

    }

    pub fn nalgebra_get_mats(&self) -> (Matrix4<f32>, Matrix4<f32>) {
        let mut mat = Matrix4::identity();

        mat *= Translation3::from_vector(Vector3::new(0.0, 0.0, -self.zoom)).to_homogeneous();
        mat *= Matrix4::from_scaled_axis(&Vector3::x() * self.ori.y)
            * Matrix4::from_scaled_axis(&Vector3::y() * self.ori.x);

        // Apply anti-OpenGL correction
        mat *= Matrix4::from_scaled_axis(-&Vector3::x() * PI / 2.0);

        mat *= Translation3::from_vector(-Vector3::new(self.focus.x, self.focus.y, self.focus.z)).to_homogeneous();

        (
            mat,
            *Perspective3::new(self.aspect_ratio, self.fov, 0.1, 10000.0).as_matrix(),
        )
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
        let p = match mats {
            Some(m) => m.0.inverted() * Vec4::new(0.0, 0.0, 0.0, 1.0),
            None => self.get_mats().0.inverted() * Vec4::new(0.0, 0.0, 0.0, 1.0)
        };

        Vec3::new(p.x, p.y, p.z)
    }

    pub fn nalgebra_get_pos(&self) -> Vec3<f32> {
        // TODO: There should be a more efficient way of doing this, but oh well
        let p = self.nalgebra_get_mats().0.try_inverse().unwrap_or(Matrix4::zeros()) * Vector4::new(0.0, 0.0, 0.0, 1.0);
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
    pub fn set_zoom(&mut self, zoom: f32) { self.zoom = zoom; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_rotation() {
        let ori: (f32, f32) = (1.0, 3.0);
        let zoom = 10.0;
        let fov = 1.5;
        let aspect_ratio = 1.618;
        let focus_nalg = Vector3::new(100.0, 100.0, 50.0);
        let focus_vek = Vec3::new(100.0, 100.0, 50.0);

        let mut mat1_nalg = Matrix4::<f32>::identity();
        let mut mat1_vek = Mat4::<f32>::identity();
        assert_eq_mat4(&mat1_nalg, &mat1_vek);

        let mat2_nalg = Matrix4::from_scaled_axis(&Vector3::x() * ori.1);
        let mat2_vek = Mat4::rotation_3d(ori.1, Vec4::<f32>::unit_x());
        assert_eq_mat4(&mat2_nalg, &mat2_vek);
        
        let mat3_nalg = Matrix4::from_scaled_axis(&Vector3::y() * ori.0);
        let mat3_vek = Mat4::rotation_3d(ori.0, Vec4::<f32>::unit_y());
        assert_eq_mat4(&mat3_nalg, &mat3_vek);

        let mat4_nalg = Matrix4::from_scaled_axis(-&Vector3::x() * PI / 2.0);
        let mat4_vek = Mat4::rotation_3d(PI / 2.0, -Vec4::unit_x());
        assert_eq_mat4(&mat4_nalg, &mat4_vek);

        let mat5_nalg = Translation3::from_vector(Vector3::new(0.0, 0.0, -zoom)).to_homogeneous();
        let mat5_vek = Mat4::<f32>::translation_3d(Vec3::new(0.0, 0.0, -zoom));
        assert_eq_mat4(&mat5_nalg, &mat5_vek);

        let mat6_nalg = Translation3::from_vector(-focus_nalg).to_homogeneous();
        let mat6_vek = Mat4::<f32>::translation_3d(-focus_vek);
        assert_eq_mat4(&mat6_nalg, &mat6_vek);

        let res_nalg = mat1_nalg * mat2_nalg * mat3_nalg * mat4_nalg * mat5_nalg * mat6_nalg;
        let res_vek = mat1_vek * mat2_vek * mat3_vek * mat4_vek * mat5_vek * mat6_vek;
        println!("{:#?}", res_nalg);
        println!("{:#?}", res_vek);
        assert_eq_mat4(&res_nalg, &res_vek);

        let persp_nalg = *Perspective3::new(aspect_ratio, fov, 0.1, 10000.0).as_matrix();
        let persp_vek = Mat4::<f32>::perspective_rh_no(fov, aspect_ratio, 0.1, 10000.0);
        println!("{:#?}", &persp_nalg);
        println!("{:#?}", &persp_vek);
        assert_eq_mat4(&persp_nalg, &persp_vek);
    }
}