use std::cmp::*;
use std::default::Default;

// External Library
use obj;
use rand;
use cgmath;
use glium;

use util::math;


pub enum TexturePBR<'a> {
    AlbedoSpecularNormalGloss(&'a glium::texture::Texture2d,
                              &'a glium::texture::Texture2d,
                              &'a glium::texture::Texture2d,
                              &'a glium::texture::Texture2d),
}
impl<'a> TexturePBR<'a> {
    fn get_texture_adress(&self) -> &glium::texture::Texture2d {
        match *self {
            TexturePBR::AlbedoSpecularNormalGloss(ref a, _, _, _) => a,
        }
    }
}

impl<'a> PartialEq for TexturePBR<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.get_texture_adress() as *const _ == other.get_texture_adress() as *const _ {
            true
        } else {
            false
        }
    }
}

impl<'a> PartialOrd for TexturePBR<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for TexturePBR<'a> {
    fn cmp(&self, other: &Self) -> Ordering {

        if self.get_texture_adress() as *const _ < other.get_texture_adress() as *const _ {
            Ordering::Less
        } else if self.get_texture_adress() as *const _ == other.get_texture_adress() as *const _ {
            Ordering::Equal
        } else {
            Ordering::Greater
        }

    }
}

impl<'a> Eq for TexturePBR<'a> {}


#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], texture: [f32; 2]) -> Vertex {
        Vertex {
            position: position,
            normal: normal,
            texture: texture,
        }
    }

    pub fn empty() -> Vertex {
        Vertex {
            position: [0.0; 3],
            normal: [0.0; 3],
            texture: [0.0; 2],
        }
    }
}
implement_vertex!(Vertex, position, normal, texture);

#[derive(Copy, Clone)]
pub struct Material {
    pub ka: [f32; 3],
    pub kd: [f32; 3],
    pub ks: [f32; 3],
}
implement_uniform_block!(Material, ka, kd, ks);

impl Material {
    pub fn new(ka: [f32; 3], kd: [f32; 3], ks: [f32; 3]) -> Material {
        Material {
            ka: ka,
            kd: kd,
            ks: ks,
        }
    }
    pub fn from(material: &obj::Material) -> Material {

        Material {
            ka: material.ka.unwrap_or([0.0; 3]),
            kd: material.kd.unwrap_or([0.0; 3]),
            ks: material.ks.unwrap_or([0.0; 3]),
        }
    }

    pub fn gen_random() -> Material {
        use rand::distributions::{IndependentSample, Range};
        let range = rand::distributions::Range::new(0.0, 1.0);
        let mut rng = rand::thread_rng();

        Material {
            ka: [range.ind_sample(&mut rng), range.ind_sample(&mut rng), range.ind_sample(&mut rng)],
            kd: [range.ind_sample(&mut rng), range.ind_sample(&mut rng), range.ind_sample(&mut rng)],
            ks: [range.ind_sample(&mut rng), range.ind_sample(&mut rng), range.ind_sample(&mut rng)],
        }
    }
}


#[derive(Copy, Clone)]
pub struct BaseUniform {
    pub model: [[f32; 4]; 4],
    pub modelview: [[f32; 4]; 4],
    pub modelviewperspective: [[f32; 4]; 4],
    pub normalmatrix: [[f32; 3]; 3],
    pub lights: Lights,
    pub ior: f32,
}

impl BaseUniform {
    pub fn new(m: &cgmath::Matrix4<f32>,
               mv: &cgmath::Matrix4<f32>,
               mvp: &cgmath::Matrix4<f32>,
               nm: [[f32; 3]; 3],
               lights: Lights,
               ior: f32)
               -> BaseUniform {
        BaseUniform {
            model: math::to_mat4(m),
            modelview: math::to_mat4(mv),
            modelviewperspective: math::to_mat4(mvp),
            normalmatrix: nm,
            lights: lights,
            ior: ior,
        }
    }
}

#[derive(Copy,Clone)]
pub struct DirectionalLight {
    pub position: [f32; 3],
    direction: [f32; 3],
    color: [f32; 3],
}
implement_uniform_block!(DirectionalLight, position, direction, color);

impl DirectionalLight {
    pub fn new(position: [f32; 3], direction: [f32; 3], color: [f32; 3]) -> DirectionalLight {
        DirectionalLight {
            position: position,
            direction: direction,
            color: color,
        }
    }
    pub fn set_position(&mut self, p: [f32; 3]) {
        self.position = p;
    }

    pub fn set_direction(&mut self, d: [f32; 3]) {
        self.direction = d;
    }

    pub fn set_color(&mut self, p: [f32; 3]) {
        self.position = p;
    }
}

#[derive(Copy, Clone)]
pub struct PointLight {
    pub pos: [f32; 3],
    _pad: f32,
    pub col: [f32; 3],
    _pad1: f32,
    pub attn: [f32; 3],
    _pad2: f32,
}
implement_uniform_block!(PointLight, pos, col, attn);

impl PointLight {
    pub fn new(position: [f32; 3],
               color: [f32; 3],
               constant: f32,
               linear: f32,
               quadratic: f32)
               -> PointLight {
        PointLight {
            pos: position,
            _pad: 0.0,
            col: color,
            _pad1: 0.0,
            attn: [constant, linear, quadratic],
            _pad2: 0.0,
        }
    }
    pub fn set_position(&mut self, p: [f32; 3]) {
        self.pos = p;
    }

    pub fn set_color(&mut self, p: [f32; 3]) {
        self.col = p;
    }
    pub fn set_attenuation(&mut self, constant: f32, linear: f32, quadratic: f32) {
        self.attn = [constant, linear, quadratic];
    }
}

#[derive(Copy, Clone)]
pub struct Lights {
    pub lights: [PointLight; 5],
}

implement_uniform_block!(Lights, lights);

impl Lights {
    pub fn gen_random() -> Lights {
        use std::mem;
        use rand::distributions::{Range, IndependentSample};
        let position_range = Range::new(-10.0, 10.0);
        let range = Range::new(0.0, 1.0);
        let mut rng = rand::thread_rng();

        let mut l: [PointLight; 5] = unsafe { mem::uninitialized() };
        for i in 0..5 {
            l[i] = PointLight::new([position_range.ind_sample(&mut rng),
                                    position_range.ind_sample(&mut rng),
                                    position_range.ind_sample(&mut rng)],
                                   [range.ind_sample(&mut rng),
                                    range.ind_sample(&mut rng),
                                    range.ind_sample(&mut rng)],
                                   1.0,
                                   0.045,
                                   0.0075)
        }
        Lights { lights: l }
    }
}

impl Default for Lights {
    fn default() -> Self {
        Lights {
            lights: [PointLight::new([4.0, 3.0, -4.0], [1.0, 1.0, 1.0], 1.0, 0.045, 0.0075),
                     PointLight::new([0.0, 3.0, 4.0], [1.0, 1.0, 1.0], 1.0, 0.045, 0.0075),
                     PointLight::new([-4.0, 3.0, -4.0], [1.0, 1.0, 1.0], 1.0, 0.045, 0.0075),
                     PointLight::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0, 0.045, 0.0075),
                     PointLight::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0, 0.045, 0.0075)],
        }
    }
}
