
use std::ops::Range;
use std::fmt::{Debug, Formatter, Result};
use std::cmp::*;

use glium;
use collision::{Frustum, Aabb3, Relation};
use cgmath;

use util::graphics::{BaseUniform, Material, Vertex, TexturePBR};
use accelerator::OctreeItem;

pub struct Group<'a> {
    range: Range<usize>,
    tex: Option<TexturePBR<'a>>,
    mat: Option<&'a Material>,
    program: &'a glium::Program,
    volume: Aabb3<f32>,
}

impl<'a> Debug for Group<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "(Program adress: {:?})", self.program)
    }
}

impl<'a> Group<'a> {
    pub fn new(r: Range<usize>,
               tex: Option<TexturePBR<'a>>,
               mat: Option<&'a Material>,
               program: &'a glium::Program,
               volume: Aabb3<f32>)
               -> Group<'a> {
        Group {
            range: r,
            tex: tex,
            mat: mat,
            program: program,
            volume: volume,
        }
    }
    pub fn get_range(&self) -> Range<usize> {
        self.range.clone()
    }

    #[inline]
    pub fn draw<S>(&self,
                   target: &mut S,
                   uniforms: BaseUniform,
                   display: &glium::Display,
                   vertex_slice: glium::vertex::VertexBufferSlice<Vertex>,
                   params: &glium::DrawParameters,
                   prim_type: &glium::index::PrimitiveType)
        where S: glium::Surface
    {

        let light_buffer = glium::uniforms::UniformBuffer::new(display, uniforms.lights).unwrap();
        let base_uniform = uniform!{
            model: uniforms.model,
            modelview: uniforms.modelview,
            modelviewperspective: uniforms.modelviewperspective,
            normalmatrix: uniforms.normalmatrix,

            ka: self.mat.unwrap().ka,
            kd: self.mat.unwrap().kd,
            ks: self.mat.unwrap().ks,
            f0: uniforms.ior,
            Block: &light_buffer,
        };

        // TODO The way that uniforms are handeled leads to this verbouse draw process_input
        // As far as I know right know, returning the uniform from a function is not possible,
        // since the type changes depending on the values. Maybe this can be done with generics..
        if let Some(ref tex) = self.tex {
            match tex {
                &TexturePBR::AlbedoSpecularNormalGloss(a, s, n, g) => {
                    let uniform = base_uniform.add("dagger_albedo", a);
                    let uniform = uniform.add("dagger_specular", s);
                    let uniform = uniform.add("dagger_normal", n);
                    let uniform = uniform.add("dagger_gloss", g);

                    target.draw(vertex_slice,
                                glium::index::NoIndices(prim_type.clone()),
                                self.program,
                                &uniform,
                                params)
                          .unwrap();
                }
            }
        } else {
            target.draw(vertex_slice,
                        glium::index::NoIndices(prim_type.clone()),
                        self.program,
                        &base_uniform,
                        params)
                  .unwrap();
        }
    }
}

impl<'a> PartialEq for Group<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.program as *const _ == other.program as *const _ {
            true
        } else {
            false
        }
    }
}

impl<'a> PartialOrd for Group<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Group<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.program as *const _ < other.program as *const _ {
            Ordering::Less
        } else if self.program as *const _ == other.program as *const _ {
            self.tex.cmp(&other.tex)
        } else {
            Ordering::Greater
        }
    }
}

impl<'a> Eq for Group<'a> {}

impl<'b, 'a: 'b> OctreeItem for &'b Group<'a> {
    fn index(&self) -> cgmath::Point3<f32> {
        self.volume.min + ((self.volume.max - self.volume.min) / 2.0)
    }

    fn is_equal(&self, other: &Self) -> bool {
        if self.volume == other.volume {
            true
        } else {
            false
        }
    }
    fn in_frustum(&self, frustum: &Frustum<f32>) -> bool {
        match frustum.contains(self.volume) {
            Relation::Out => false,
            _ => true,
        }
    }
}
