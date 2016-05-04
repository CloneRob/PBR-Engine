#[allow(dead_code)]
// Standard Library
use std::collections::HashMap;
use std::path::Path;
use std::marker::PhantomData;

// External libraries
use glium;
use collision::Aabb3;

// Importing modules of this project
use assets::Drawable;
use assets::group::Group;
use assets::{loader, build_aabb};
use shader;
use util::graphics::{Vertex, Material, TexturePBR, BaseUniform};


pub struct Asset<'a> {
    name: String,
    volume: Aabb3<f32>,
    vbo: glium::vertex::VertexBuffer<Vertex>,
    prim_type: glium::index::PrimitiveType,
    param: glium::DrawParameters<'a>,
    group: Vec<Group<'a>>,
}

impl<'a> Asset<'a> {
    pub fn get_volume(&self) -> &Aabb3<f32> {
        &self.volume
    }

    pub fn get_groups(&self) -> &[Group] {
        &self.group[..]
    }
    pub fn groups_ref(&self) -> Vec<&Group> {
        let mut g_ref = Vec::with_capacity(self.group.len());
        for g in self.group.iter() {
            g_ref.push(g);
        }
        g_ref
    }

    pub fn print_groups(&self) {
        for g in self.group.iter() {
            println!("{:?}", g);
        }
    }

    pub fn num_o_groups(&self) -> usize {
        self.group.len()
    }

    fn sort_group(&mut self) {
        self.group.sort();
    }
    pub fn print_vb(&self) {
        println!("{:?}", self.vbo.len());
    }
    pub fn get_vbo(&self) -> &glium::VertexBuffer<Vertex> {
        &self.vbo
    }
}

impl<'a> Drawable for Asset<'a> {
    #[inline]
    fn draw<S>(&self, target: &mut S, display: &glium::Display, uniforms: BaseUniform)
        where S: glium::Surface
    {
        for g in self.group.iter() {
            g.draw(target,
                   uniforms,
                   display,
                   self.vbo.slice(g.get_range()).unwrap(),
                   &self.param,
                   &self.prim_type);
        }
    }

    #[inline]
    fn draw_group<S>(&self,
                     target: &mut S,
                     display: &glium::Display,
                     uniforms: BaseUniform,
                     group: &Group)
        where S: glium::Surface
    {
        group.draw(target,
                   uniforms,
                   display,
                   self.vbo.slice(group.get_range()).unwrap(),
                   &self.param,
                   &self.prim_type);
    }
}

pub struct AssetLoader<'a, T: 'a> {
    name: String,
    display: &'a glium::Display,
    vertex_data: Vec<Vertex>,
    group: Vec<Group<'a>>,
    param: Option<glium::DrawParameters<'a>>,
    prim_type: Option<glium::index::PrimitiveType>,
    volume: Aabb3<f32>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: 'a> AssetLoader<'a, T> {
    pub fn new(display: &'a glium::Display,
               path: &Path,
               texture_map: &'a HashMap<String, glium::texture::Texture2d>,
               material_map: &'a HashMap<String, Material>,
               program: &'a shader::Program)
               -> AssetLoader<'a, T> {
        let (vertex, group) = loader(path, texture_map, material_map, program);
        AssetLoader {
            name: "".to_string(),
            display: display,
            volume: build_aabb(&vertex),
            vertex_data: vertex,
            group: group,
            param: None,
            prim_type: None,
            phantom: PhantomData,
        }
    }
    pub fn custom(display: &'a glium::Display,
                  name: String,
                  vertex_data: Vec<Vertex>,
                  material: &'a Material,
                  texture_albedo: Option<&'a glium::texture::Texture2d>,
                  texture_specular: Option<&'a glium::texture::Texture2d>,
                  texture_normal: Option<&'a glium::texture::Texture2d>,
                  texture_gloss: Option<&'a glium::texture::Texture2d>,
                  program: &'a glium::Program)
                  -> AssetLoader<'a, T> {
        let vol = build_aabb(&vertex_data);
        let range = 0..vertex_data.len();
        let al = match (texture_albedo,
                        texture_specular,
                        texture_normal,
                        texture_gloss) {
            (Some(a), Some(s), Some(n), Some(g)) => {
                AssetLoader {
                    name: name,
                    display: display,
                    volume: vol.clone(),
                    vertex_data: vertex_data,
                    group: vec![Group::new(range,
                                           Some(TexturePBR::AlbedoSpecularNormalGloss(a, s, n, g)),
                                           Some(material),
                                           program,
                                           vol)],
                    param: None,
                    prim_type: None,
                    phantom: PhantomData,
                }
            }
            _ => {
                AssetLoader {
                    name: name,
                    display: display,
                    volume: vol.clone(),
                    vertex_data: vertex_data,
                    group: vec![Group::new(range, None, Some(material), program, vol)],
                    param: None,
                    prim_type: None,
                    phantom: PhantomData,
                }
            }
        };
        al
    }

    pub fn name(mut self, name: String) -> AssetLoader<'a, T> {
        self.name = name;
        self
    }
    pub fn param(mut self, param: glium::DrawParameters<'a>) -> AssetLoader<'a, T> {
        self.param = Some(param);
        self
    }
    pub fn prim_type(mut self, pt: glium::index::PrimitiveType) -> AssetLoader<'a, T> {
        self.prim_type = Some(pt);
        self
    }
    pub fn volume(mut self, volume: Aabb3<f32>) -> AssetLoader<'a, T> {
        self.volume = volume;
        self
    }
    pub fn load(mut self) -> Asset<'a> {
        if let None = self.param {
            self.param = Some(glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                    write: true,
                    ..Default::default()
                },
                // backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                ..Default::default()
            });
        }
        if let None = self.prim_type {
            self.prim_type = Some(glium::index::PrimitiveType::TrianglesList);
        }
        let mut asset = Asset {
            name: self.name,
            volume: self.volume,
            vbo: glium::vertex::VertexBuffer::immutable(self.display, &self.vertex_data).unwrap(),
            prim_type: self.prim_type.unwrap(),
            param: self.param.unwrap(),
            group: self.group,
        };
        asset.sort_group();
        asset
    }
}
