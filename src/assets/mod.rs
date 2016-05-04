//Standard Library
use std::fs;
use std::fs::File;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

//External Library
use glium;
use glium::{Display, Surface};
use collision::Aabb3;
use image;

use shader;
use assets::group::{Group};
use util::graphics::{Vertex, BaseUniform, Material, TexturePBR};

pub mod asset;
pub mod instance;
pub mod group;


pub trait Drawable {
    fn draw<S>(&self, target: &mut S,display: &glium::Display, uniforms: BaseUniform)
        where S: Surface;
    fn draw_group<S>(&self, target: &mut S,display: &glium::Display, uniforms: BaseUniform, group: &Group)
        where S: Surface;
}

#[inline]
fn build_aabb(vertex_data: &[Vertex]) -> Aabb3<f32> {
    use std::cmp::Ordering;
    use cgmath;

    let mut pos_x = Vec::new();
    let mut pos_y = Vec::new();
    let mut pos_z = Vec::new();

    for p in vertex_data.iter() {
        let tmp = &p.position[..];
        pos_x.push(tmp[0]);
        pos_y.push(tmp[1]);
        pos_z.push(tmp[2]);
    }

    pos_x.sort_by(|a, b| a.partial_cmp(&b).unwrap_or(Ordering::Less));
    pos_y.sort_by(|a, b| a.partial_cmp(&b).unwrap_or(Ordering::Less));
    pos_z.sort_by(|a, b| a.partial_cmp(&b).unwrap_or(Ordering::Less));


    Aabb3::new(cgmath::Point3::new(pos_x[0], pos_y[0], pos_z[0]),
               cgmath::Point3::new(pos_x.last().unwrap().clone(),
                                   pos_y.last().unwrap().clone(),
                                   pos_z.last().unwrap().clone()))
}

#[inline]
pub fn loader<'b, 'a: 'b>(path: &Path,
                          texture_map: &'a HashMap<String, glium::texture::Texture2d>,
                          material_map: &'a HashMap<String, Material>,
                          program: &'a shader::Program)
                          -> (Vec<Vertex>, Vec<Group<'b>>) {
    use obj;
    use genmesh;

    let data = obj::load::<genmesh::Polygon<obj::IndexTuple>>(path).unwrap();
    let mut vertex_data = Vec::new();
    let mut groups = Vec::new();
    let mut group_index = 0;

    for object in data.object_iter() {
        for group in object.group_iter() {
            let mut group_len = 0;
            for g in group.indices().iter() {
                match g {
                    &genmesh::Polygon::PolyTri(genmesh::Triangle{x: v1, y: v2, z: v3}) => {
                        group_len += 3;
                        for v in [v1, v2, v3].iter() {
                            let position = data.position()[v.0];
                            let texture = v.1.map(|index| data.texture()[index]);
                            let normal = v.2.map(|index| data.normal()[index]);

                            let texture = texture.unwrap_or([0.0, 0.0]);
                            let normal = normal.unwrap_or([0.0, 0.0, 0.0]);

                            vertex_data.push(Vertex::new(position, normal, texture));
                        }
                    }
                    &genmesh::Polygon::PolyQuad(genmesh::Quad{x: v1, y: v2, z: v3, w: v4}) => {
                        group_len += 6;
                        for v in [v1, v2, v3, v3, v4, v1].iter() {
                            let position = data.position()[v.0];
                            let texture = v.1.map(|index| data.texture()[index]);
                            let normal = v.2.map(|index| data.normal()[index]);

                            let texture = texture.unwrap_or([0.0, 0.0]);
                            let normal = normal.unwrap_or([0.0, 0.0, 0.0]);

                            vertex_data.push(Vertex::new(position, normal, texture));
                        }
                    }
                }
            }
            let group_range = group_index..group_index + group_len;
            let albedo = texture_map.get("Dagger_Albedo.png");
            let specular = texture_map.get("Dagger_Specular.png");
            let normal = texture_map.get("Dagger_Normals.png");
            let gloss = texture_map.get("Dagger_Gloss.png");
            println!("{:?} {:?} {:?}  {:?}", albedo, specular, normal, gloss);

            let (program_ref, texture) = match (albedo, specular, normal, gloss) {
                (Some(a), Some(s), Some(n), Some(g)) => {
                    println!("found some textures");
                    (&program.ambient_diffuse_bump, Some(TexturePBR::AlbedoSpecularNormalGloss(a, s, n, g)))
                },
                _ => (&program.none, None)
            };
            groups.push(Group::new(group_range.clone(),
                                   texture,
                                   material_map.get("base_material"),
                                   program_ref,
                                  build_aabb(&vertex_data[group_range])));
            group_index += group_len;
        }
    }
    (vertex_data, groups)
}

pub fn image_format(path: &PathBuf) -> Option<image::ImageFormat> {
    let file_type = path.extension().expect("path.extensio").to_str().expect("path.extension.to_str()").clone();
    match file_type {
        "bmp" | "BMP" => Some(image::ImageFormat::BMP),
        "jpg" | "JPG" => Some(image::ImageFormat::JPEG),
        "png" | "PNG" => Some(image::ImageFormat::PNG),
        "tga" | "TGA" => Some(image::ImageFormat::TGA),
        _ => {
            None
        }
    }
}

pub fn build_texture_map(display: &glium::Display, path: &Path) -> HashMap<String, glium::texture::Texture2d> {
    let paths = fs::read_dir(path)
                    .expect("Could not read directory in build_texture_map");

    let mut texture_map = HashMap::new();
    for p in paths {
        let path = p.expect("Panic in build_texture_map on path extraction").path();
        // TODO Rework the types to remove this abomination
        let file_name = path.file_name()
                            .expect("Panic on conversion to file name string")
                            .to_str()
                            .unwrap()
                            .to_string();

        if let Some(format) = image_format(&path) {
            let image_file = File::open(path).expect("Panic on texture init for material with ");
            let image = image::load(image_file, format).expect("Image load").to_rgba();
            let image_dim = image.dimensions();
            let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dim);
            let texture = glium::texture::Texture2d::new(display, image).unwrap();
            texture_map.insert(file_name, texture);
        }
    }
    texture_map
}

pub fn build_material_map(path: &Path) -> HashMap<String, Material> {
    use std::io::BufReader;
    use obj;

    let paths = fs::read_dir(path)
                    .expect("Could not read dir in build_material_map");

    let mut material_map = HashMap::new();
    material_map.insert("base_material".into(),
                        Material {
                            ka: [0.2, 0.0, 0.0],
                            kd: [0.7, 0.0, 0.0],
                            ks: [1.0, 1.0, 1.0],
                        });
    for p in paths {

        let path = p.expect("Panic in build_texture_map on path extraction").path();
        let file = File::open(path)
                       .expect("Path supplied to build_material_map(path: &Path) was wrong");

        let mut reader = BufReader::new(file);
        let data = obj::Mtl::load(&mut reader);
        for m in data.materials.iter() {
            material_map.insert(m.name.clone(), Material::from(m));
        }
    }
    material_map
}
