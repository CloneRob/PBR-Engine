extern crate glutin;

use std::cell::RefCell;
use std::f64::consts::PI;
use std::f32;
use std::fmt;
use rand;

use glium;
use collision::{Frustum, Relation, Aabb3};
use cgmath::*;

use util::math::{rotate_matrix, rotation_matrix};
use util::graphics::BaseUniform;
use accelerator::OctreeItem;
use assets::asset::Asset;
use assets::group::Group;
use assets::Drawable;

pub struct InstanceLoader<'b, 'a: 'b> {
    asset: &'b Asset<'a>,
    volume: Option<Aabb3<f32>>,
    to_world: Option<Matrix4<f32>>,
}

impl<'b, 'a: 'b> InstanceLoader<'b, 'a> {
    pub fn new(asset: &'b Asset<'a>) -> InstanceLoader<'b, 'a> {
        InstanceLoader {
            asset: asset,
            volume: None,
            to_world: None,
        }
    }
    pub fn scale(mut self, f: f32) -> InstanceLoader<'b, 'a> {
        let mut m: Matrix4<f32> = SquareMatrix::one();
        m.x.x = f;
        m.y.y = f;
        m.z.z = f;
        if let Some(mat) = self.to_world {
            m = &m * &mat;
        }
        self.to_world = Some(m);
        // println!("after scaling:\n{:?}", self.to_world);
        self
    }
    pub fn translate(mut self, t: Vector3<f32>) -> InstanceLoader<'b, 'a> {
        if let Some(vol) = self.volume {
            vol.min + t;
            vol.max + t;
        } else {
            self.volume = Some(Aabb3::new(self.asset.get_volume().min + t,
                                          self.asset.get_volume().max + t))
        }
        if let Some(ref mut matrix) = self.to_world {
            matrix.w.x = t.x;
            matrix.w.y = t.y;
            matrix.w.z = t.z;
        } else {
            self.to_world = Some(Matrix4::from_translation(t));
        }
        self
    }

    pub fn rand_rotate(self) -> InstanceLoader<'b, 'a> {
        use rand::distributions::{IndependentSample, Range};

        let r = Range::new(-PI as f32, PI as f32);
        let a = Range::new(-1.0, 1.0);
        let mut rng = rand::thread_rng();

        let axis = Vector3::new(a.ind_sample(&mut rng),
                                a.ind_sample(&mut rng),
                                a.ind_sample(&mut rng));
        let axis = axis.normalize();

        self.rotate(Rad { s: r.ind_sample(&mut rng) }, axis)
    }

    pub fn rotate(mut self, angle: Rad<f32>, axis: Vector3<f32>) -> InstanceLoader<'b, 'a> {
        let rotate = rotation_matrix(angle, axis);
        if let Some(m) = self.to_world {
            let translation_vector = Vector3::new(m.w.x, m.w.y, m.w.z);

            let mut rotate_homogen = Matrix4::from(rotate);
            rotate_homogen.w.x = translation_vector.x;
            rotate_homogen.w.y = translation_vector.x;
            rotate_homogen.w.z = translation_vector.z;

            let translation = Matrix4::from_translation(translation_vector);

            self.to_world = Some(&translation * &(&rotate_homogen * &m));
        } else {
            self.to_world = Some(Matrix4::from(rotate));
        }
        self
    }

    pub fn load(mut self) -> AssetInstance<'b, 'a> {
        if let None = self.volume {
            self.volume = Some(self.asset.get_volume().clone());
        }
        if let None = self.to_world {
            self.to_world = Some(SquareMatrix::one());
        }
        AssetInstance {
            asset: self.asset,
            volume: self.volume.unwrap(),
            to_world: self.to_world.unwrap(),
        }
    }
}

#[allow(dead_code)]
pub struct AssetInstance<'b, 'a: 'b> {
    asset: &'b Asset<'a>,
    volume: Aabb3<f32>,
    to_world: Matrix4<f32>,
}

impl<'b, 'a: 'b> OctreeItem for &'b AssetInstance<'b, 'a> {
    fn index(&self) -> Point3<f32> {
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

// TODO Implement this properly
impl<'b, 'a: 'b> fmt::Debug for &'b AssetInstance<'b, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(AssetInstance\n)")
    }
}

impl<'a, 'b> AssetInstance<'a, 'b> {
    pub fn set_to_world(&mut self, _to_world: Matrix4<f32>) {
        self.to_world = _to_world;
    }
    pub fn get_to_world(&self) -> &Matrix4<f32> {
        &self.to_world
    }
    pub fn get_volume(&self) -> &Aabb3<f32> {
        &self.volume
    }
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.to_world.w.x += x;
        self.to_world.w.y += y;
        self.to_world.w.z += z;
    }

    #[inline]
    pub fn rotate_angles(&mut self,
                         vertical_offset: Rad<f32>,
                         horizontal_offset: Rad<f32>,
                         point: Vector3<f32>) {
        let q_vertical = Quaternion::from_axis_angle(Vector3::unit_x(), vertical_offset);
        let q_horizontal = Quaternion::from_axis_angle(Vector3::unit_y(), horizontal_offset);

        let matrix = Matrix4::from(Matrix3::from(&q_vertical * &q_horizontal));

        let mut t0 = Matrix4::one();
        t0.w.x = -point.x;
        t0.w.y = -point.y;
        t0.w.z = -point.z;

        let mut t1 = Matrix4::one();
        t1.w.x = point.x;
        t1.w.y = point.y;
        t1.w.z = point.z;

        self.to_world = &t1 * &(&matrix * &(&t0 * &self.to_world));

    }

    #[inline]
    pub fn rotate(&mut self, matrix: Matrix4<f32>) {

        let mut t0 = Matrix4::one();
        t0.w.x = -self.to_world.w.x;
        t0.w.y = -self.to_world.w.y;
        t0.w.z = -self.to_world.w.z;

        let mut t1 = Matrix4::one();
        t1.w.x = self.to_world.w.x;
        t1.w.y = self.to_world.w.y;
        t1.w.z = self.to_world.w.z;

        self.to_world = &t1 * &(&matrix * &(&t0 * &self.to_world));

    }
    pub fn update(&mut self, entity: &Entity) {
        // let mut transform;
        if entity.moveing.up {
            self.translate(0.0, 0.1, 0.0);
        }
        if entity.moveing.down {
            self.translate(0.0, -0.1, 0.0);
        }
        if entity.moveing.left {
            self.translate(-0.1, 0.0, 0.0);
        }
        if entity.moveing.right {
            self.translate(0.1, 0.0, 0.0);
        }
        if entity.moveing.forward {
            self.translate(0.0, 0.0, -0.1);
        }
        if entity.moveing.backward {
            self.translate(0.0, 0.0, 0.1);
        }

        let rad = Rad { s: 0.5 * f32::consts::PI / 180.0 };
        let rad_0 = Rad { s: 0.0 };
        if entity.rotating.pitch_u {
            let r = Matrix4::from(Matrix3::from_angle_x(Rad { s: 0.01 }));
            self.rotate(r);
        }
        if entity.rotating.pitch_d {
            let r = Matrix4::from(Matrix3::from_angle_x(Rad { s: -0.01 }));
            self.rotate(r);
        }
        if entity.rotating.yaw_l {
            let r = Matrix4::from(Matrix3::from_angle_y(Rad { s: 0.01 }));
            self.rotate(r);
        }
        if entity.rotating.yaw_r {
            let r = Matrix4::from(Matrix3::from_angle_y(Rad { s: -0.01 }));
            self.rotate(r);
        }
        if entity.reset {
            self.set_to_world(Matrix4::one());
        }
    }
}
pub struct Moveing {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    forward: bool,
    backward: bool,
}
pub struct Rotating {
    pitch_u: bool,
    pitch_d: bool,
    yaw_l: bool,
    yaw_r: bool,
    roll_l: bool,
    roll_r: bool,
}

pub struct Entity {
    pub moveing: Moveing,
    pub rotating: Rotating,
    pub reset: bool,
}

impl Entity {
    pub fn new() -> Entity {
        Entity {
            moveing: Moveing {
                up: false,
                down: false,
                left: false,
                right: false,
                forward: false,
                backward: false,
            },
            rotating: Rotating {
                pitch_u: false,
                pitch_d: false,
                yaw_l: false,
                yaw_r: false,
                roll_l: false,
                roll_r: false,
            },
            reset: false,
        }
    }
    pub fn process_input(&mut self, event: &glutin::Event) {
        match event {
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Numpad4)) => {
                self.rotating.yaw_l = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Numpad4)) => {
                self.rotating.yaw_l = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Numpad6)) => {
                self.rotating.yaw_r = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Numpad6)) => {
                self.rotating.yaw_r = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Numpad8)) => {
                self.rotating.pitch_u = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Numpad8)) => {
                self.rotating.pitch_u = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Numpad2)) => {
                self.rotating.pitch_d = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Numpad2)) => {
                self.rotating.pitch_d = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::T)) => {
                self.moveing.forward = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::T)) => {
                self.moveing.forward = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::G)) => {
                self.moveing.backward = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::G)) => {
                self.moveing.backward = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::F)) => {
                self.moveing.left = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::F)) => {
                self.moveing.left = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::H)) => {
                self.moveing.right = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::H)) => {
                self.moveing.right = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::R)) => {
                self.moveing.down = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::R)) => {
                self.moveing.down = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Z)) => {
                self.moveing.up = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Z)) => {
                self.moveing.up = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::B)) => {
                self.reset = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::B)) => {
                self.reset = false;
            }
            _ => {}
        }
    }
}

impl<'a, 'b> Drawable for AssetInstance<'a, 'b> {
    fn draw<S>(&self, target: &mut S, display: &glium::Display, uniforms: BaseUniform)
        where S: glium::Surface
    {
        self.asset.draw(target, display, uniforms);
    }


    fn draw_group<S>(&self,
                     target: &mut S,
                     display: &glium::Display,
                     uniforms: BaseUniform,
                     group: &Group)
        where S: glium::Surface
    {
        self.asset.draw_group(target, display, uniforms, group);
    }
}
// ***************************************************************************
// ***************************************************************************
// ***************************************************************************
//              Test Instance and Loader
// ***************************************************************************
// ***************************************************************************
// ***************************************************************************
pub struct InstanceLoader2<'b, 'a: 'b> {
    asset: &'b Asset<'a>,
    volume: Option<Aabb3<f32>>,
    to_world: Option<Matrix4<f32>>,
}

impl<'b, 'a: 'b> InstanceLoader2<'b, 'a> {
    pub fn new(asset: &'b Asset<'a>) -> InstanceLoader2<'b, 'a> {
        InstanceLoader2 {
            asset: asset,
            volume: None,
            to_world: None,
        }
    }
    pub fn scale(mut self, f: f32) -> InstanceLoader2<'b, 'a> {
        if let Some(mut mat) = self.to_world {
            mat.x.x = f;
            mat.y.y = f;
            mat.z.z = f;
        } else {
            let mut m: Matrix4<f32> = SquareMatrix::one();
            m.x.x = f;
            m.y.y = f;
            m.z.z = f;
            self.to_world = Some(m);
        }
        self
    }
    pub fn translation(mut self, t: Vector3<f32>) -> InstanceLoader2<'b, 'a> {
        if let Some(vol) = self.volume {
            vol.min + t;
            vol.max + t;
        } else {
            self.volume = Some(Aabb3::new(self.asset.get_volume().min + t,
                                          self.asset.get_volume().max + t))
        }
        if let Some(mut matrix) = self.to_world {
            matrix.x.w = t.x;
            matrix.y.w = t.y;
            matrix.z.w = t.y;
        } else {
            self.to_world = Some(Matrix4::from_translation(t));
        }
        self
    }

    pub fn load(mut self) -> AssetInstance2<'b, 'a> {
        if let None = self.volume {
            self.volume = Some(self.asset.get_volume().clone());
        }
        if let None = self.to_world {
            self.to_world = Some(SquareMatrix::one());
        }

        AssetInstance2 {
            asset: self.asset,
            volume: RefCell::new(self.volume.unwrap()),
            to_world: RefCell::new(self.to_world.unwrap()),
        }
    }
}
#[allow(dead_code)]
pub struct AssetInstance2<'b, 'a: 'b> {
    asset: &'b Asset<'a>,
    volume: RefCell<Aabb3<f32>>,
    to_world: RefCell<Matrix4<f32>>,
}
impl<'a, 'b> AssetInstance2<'a, 'b> {
    pub fn set_to_world(&mut self, to_world: Matrix4<f32>) {
        self.to_world = RefCell::new(to_world);
    }

    pub fn get_to_world(&self) -> Matrix4<f32> {
        *self.to_world.borrow()
    }

    pub fn get_volume(&self) -> Aabb3<f32> {
        *self.volume.borrow()
    }

    pub fn translate(&self, x: f32, y: f32, z: f32) {
        self.to_world.borrow_mut().x.w += x;
        self.to_world.borrow_mut().y.w += y;
        self.to_world.borrow_mut().z.w += z;
    }
}
impl<'a, 'b> Drawable for AssetInstance2<'a, 'b> {
    fn draw<S>(&self, target: &mut S, display: &glium::Display, uniforms: BaseUniform)
        where S: glium::Surface
    {
        self.asset.draw(target, display, uniforms);
    }


    fn draw_group<S>(&self,
                     target: &mut S,
                     display: &glium::Display,
                     uniforms: BaseUniform,
                     group: &Group)
        where S: glium::Surface
    {
        self.asset.draw_group(target, display, uniforms, group);
    }
}

impl<'b, 'a: 'b> OctreeItem for &'b AssetInstance2<'b, 'a> {
    fn index(&self) -> Point3<f32> {
        self.volume.borrow().min + ((self.volume.borrow().max - self.volume.borrow().min) / 2.0)
    }

    fn is_equal(&self, other: &Self) -> bool {
        if *self.volume.borrow() == *other.volume.borrow() {
            true
        } else {
            false
        }
    }
    fn in_frustum(&self, frustum: &Frustum<f32>) -> bool {
        match frustum.contains(*self.volume.borrow()) {
            Relation::Out => false,
            _ => true,
        }
    }
}

// TODO Implement this properly
impl<'b, 'a: 'b> fmt::Debug for &'b AssetInstance2<'b, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(AssetInstance\n)")
    }
}
