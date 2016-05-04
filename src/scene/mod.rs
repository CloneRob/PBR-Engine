
use std::f32;
use std::cell::RefCell;
use glium;
use glium::glutin::{Event, ElementState, VirtualKeyCode};
use cgmath;
use cgmath::{Vector3, Rad};
use rand;
use time;

use assets::{asset, instance};
use camera;
use util::graphics::Lights;

pub enum Action {
    Stop,
    Continue,
    Input(Vec<Event>),
}

pub enum Action2<'a> {
    Stop,
    Input(glium::backend::glutin_backend::PollEventsIter<'a>),
}

type Entity = usize;

pub struct Scene<'b, 'a: 'b> {
    instances: Vec<instance::AssetInstance2<'b, 'a>>,
    entity: Entity,
    pub camera: RefCell<camera::Camera2>,
    pub lights: RefCell<Lights>,
}

impl<'b, 'a: 'b> Scene<'b, 'a> {
    #[inline]
    pub fn start_loop<F>(&self, mut callback: F)
        where F: FnMut() -> Action2<'b>
    {
        use std::time::Duration;

        loop {
            let now = time::PreciseTime::now();
            match callback() {
                Action2::Stop => break,
                Action2::Input(e) => {
                    for ev in e {
                        self.process_input2(ev);
                    }
                }
            };
            while now.to(time::PreciseTime::now()) < time::Duration::milliseconds(15) {

            }
        }
    }

    pub fn camera_view(&self) -> cgmath::Matrix4<f32> {
        self.camera.borrow().view()
    }
    pub fn camera_perspective(&self) -> cgmath::Matrix4<f32> {
        self.camera.borrow().perspective()
    }

    pub fn space(planets: &'a [asset::Asset<'a>], ship: &'b asset::Asset<'a>) -> Scene<'b, 'a> {
        use rand::distributions::{IndependentSample, Range};
        let scale = Range::new(0.0, 1.05);
        let pos = Range::new(-400.0, 400.0);
        let mut rng = rand::thread_rng();

        let mut instance_list = Vec::with_capacity(15000);
        instance_list.push(instance::InstanceLoader2::new(ship).scale(0.001).load());
        for p in planets.iter() {
            for _ in 0..1 {
                let random_pos = Vector3::new(pos.ind_sample(&mut rng),
                                              pos.ind_sample(&mut rng),
                                              pos.ind_sample(&mut rng));
                instance_list.push(instance::InstanceLoader2::new(p)
                                       .scale(scale.ind_sample(&mut rng))
                                       .translation(random_pos)
                                       .load());
            }
        }

        Scene {
            instances: instance_list,
            entity: 0,
            camera: RefCell::new(camera::Camera2::new(0.01, 0.01)),
            lights: RefCell::new(Lights::gen_random()),
        }
    }
    pub fn instances(&self) -> &[instance::AssetInstance2<'b, 'a>] {
        &self.instances[..]
    }
    pub fn process_input2(&self, event: Event) {
        const RAD0: Rad<f32> = Rad { s: 0.5 * f32::consts::PI / 180.0 };
        const RAD1: Rad<f32> = Rad { s: 0.0 };
        let mut translate = Vector3::new(0.0, 0.0, 0.0);
        match event {
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::E)) => {
                translate = translate + self.camera.borrow_mut().up();
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Q)) => {
                translate = translate - self.camera.borrow_mut().up();
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                translate = translate + self.camera.borrow_mut().forward();
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                translate = translate - self.camera.borrow_mut().forward();
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::D)) => {
                translate = translate + self.camera.borrow_mut().right();
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                translate = translate - self.camera.borrow_mut().right();
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Up)) => {
                self.camera.borrow_mut().offset_orientation(RAD0, RAD1);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Down)) => {
                self.camera.borrow_mut().offset_orientation(-RAD0, RAD1);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Left)) => {
                self.camera.borrow_mut().offset_orientation(RAD1, -RAD0);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Right)) => {
                self.camera.borrow_mut().offset_orientation(RAD1, RAD0);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => {
                self.camera.borrow_mut().reset_position();
                self.camera.borrow_mut().reset_orientation();
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Key1)) => {
                self.camera.borrow_mut().update_movespeed(0.01);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Key2)) => {
                self.camera.borrow_mut().update_movespeed(-0.01);
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::H)) => {
                self.instances[self.entity].translate(-0.01, 0.0, 0.0)
            }
            Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::J)) => {
                self.instances[self.entity].translate(0.01, 0.0, 0.0)
            }
            _ => {}
        }
        self.camera.borrow_mut().offset_position(translate);
    }
}
