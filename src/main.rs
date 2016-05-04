#![allow(dead_code)]
#[macro_use]
extern crate glium;
extern crate genmesh;
extern crate obj;
extern crate collision;
extern crate cgmath;
extern crate image;
extern crate time;
extern crate rand;

//Modules of this project
mod shader;
mod assets;
mod util;
mod camera;
mod accelerator;
mod scene;

use glium::Surface;
use glium::glutin::{Event, ElementState, VirtualKeyCode};
use cgmath::*;

use assets::{Drawable, asset, instance};
use util::graphics::BaseUniform;

fn main() {
    use glium::{DisplayBuild, Surface};
    use std::path::Path;

    use cgmath::*;

    let disp = glium::glutin::WindowBuilder::new()
                      .with_dimensions(1600, 900)
                      .with_depth_buffer(24)
                      //.with_multisampling(4)
                      //.with_srgb(Some(true))
                      .build_glium();

    let display = if let Ok(res) = disp {
        res
    } else {
        panic!("Something goes wrong here");
    };


    let dagger_path = Path::new("/home/robert/Projects/rust/pbr/src/resource/test/Dagger.obj");
    let texture_path = Path::new("/home/robert/Projects/rust/pbr/src/resource/texture");
    let material_path = Path::new("/home/robert/Projects/rust/pbr/src/resource/mtl");

    let texture_map = assets::build_texture_map(&display, texture_path);

    let material_map = assets::build_material_map(material_path);


    println!("Creating Program map");
    let program_map = shader::program_map(&display);


    println!("\nStarting AssetLoader");

    let dagger = asset::AssetLoader::<f32>::new(&display, dagger_path, &texture_map, &material_map, program_map.get("cooktorrance").unwrap()).load();
    let mut dagger_instance = instance::InstanceLoader::new(&dagger).load();

    let mut entity = instance::Entity::new();
    let mut lights = util::graphics::Lights::default();
    let mut camera = camera::Camera::new();
    let mut t:f32 = 0.0;
    let mut ior = 0.72;
    let two_pi:f32 = 2.0 * 3.14159265358979323846264338;
    util::start_loop(|| {
    // Passed parameter represents movespeed
        util::timer(|| {
            camera.update(1.0);
            dagger_instance.update(&entity);
            let perspective = camera.perspective();
            let view = camera.view();

            let mut target = display.draw();
            target.clear_color_and_depth((1.0,1.0,1.0,1.0), 1.0);

            let rotation = cgmath::Matrix3::from_angle_y(Rad{s: t});

            //let model = &cgmath::Matrix4::from(rotation) * dagger_instance.get_to_world();
            let model = dagger_instance.get_to_world();
            let model_view = &view * model;
            let model_view_perspective = &perspective * &model_view;

            let uniform = BaseUniform::new(
                &model,
                &model_view,
                &model_view_perspective,
                util::math::from_mat4(&model_view),
                lights,
                ior,
            );
            dagger_instance.draw(&mut target, &display, uniform);
            if t > two_pi {
                t = 0.0
            } else {
                t += 0.005;
            }
            target.finish().unwrap();
        });
        for event in display.poll_events() {
            match event {
                Event::Closed => return util::Action::Stop,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Key1)) => {
                    ior += 0.05;
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Key2)) => {
                    ior -= 0.05;
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => {
                    lights = util::graphics::Lights::gen_random();
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::X)) => {
                    lights = util::graphics::Lights::default();
                }
                x => {
                    //TODO move this into one function
                    camera.process_input(&x);
                    entity.process_input(&x);
                },
            }
        }
        util::Action::Continue
    });
}
