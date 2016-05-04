extern crate glutin;
extern crate cgmath;

use std::f32;

use cgmath::*;

struct Moveing {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    forward: bool,
    backward: bool,
}
struct Rotating {
    pitch_u: bool,
    pitch_d: bool,
    yaw_l: bool,
    yaw_r: bool,
    roll_l: bool,
    roll_r: bool,
}

pub struct Camera {
    position: Vector3<f32>,
    pub perspective: PerspectiveFov<f32, Rad<f32>>,
    vertical_angle: Rad<f32>,
    horizontal_angle: Rad<f32>,

    movement: Moveing,
    rotation: Rotating,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vector3::new(0.0, 0.0, 0.0),
            perspective: PerspectiveFov {
                fovy: Rad { s: 0.69 },
                aspect: 16.0 / 9.0,
                near: 0.1,
                far: 2000.0,
            },
            vertical_angle: Rad { s: 0.0 },
            horizontal_angle: Rad { s: 0.0 },

            movement: Moveing {
                up: false,
                down: false,
                left: false,
                right: false,
                forward: false,
                backward: false,
            },

            rotation: Rotating {
                pitch_u: false,
                pitch_d: false,
                yaw_l: false,
                yaw_r: false,
                roll_l: false,
                roll_r: false,
            },
        }
    }

    pub fn set_position(&mut self, position: (f32, f32, f32)) {
        self.position = Vector3::from(position);
    }

    pub fn set_fov(&mut self, fov: Rad<f32>) {
        self.perspective.fovy = fov;
    }

    pub fn offset_position(&mut self, offset: Vector3<f32>) {
        self.position = self.position + offset;
    }

    pub fn offset_orientation(&mut self, up_angle: Rad<f32>, right_angle: Rad<f32>) {
        self.vertical_angle = self.vertical_angle - up_angle;
        self.horizontal_angle = self.horizontal_angle + right_angle;
    }

    pub fn orientation(&self) -> Matrix3<f32> {
        let q_vertical = Quaternion::from_axis_angle(Vector3::unit_x(), self.vertical_angle);
        let q_horizontal = Quaternion::from_axis_angle(Vector3::unit_y(), self.horizontal_angle);
        Matrix3::from(&q_vertical * &q_horizontal)
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        &self.perspective() * &self.view()
    }

    pub fn forward(&self) -> Vector3<f32> {
        &self.orientation().transpose() * &Vector3::new(0.0, 0.0, -1.0)
    }

    pub fn right(&self) -> Vector3<f32> {
        &self.orientation().transpose() * &Vector3::new(1.0, 0.0, 0.0)
    }

    pub fn up(&self) -> Vector3<f32> {
        &self.orientation().transpose() * &Vector3::new(0.0, 1.0, 0.0)
    }

    pub fn view(&self) -> Matrix4<f32> {
        &Matrix4::from(self.orientation()) * &Matrix4::from_translation(-self.position)
    }

    pub fn perspective(&self) -> Matrix4<f32> {
        Matrix4::from(self.perspective)
    }

    pub fn to_perspective(&self) -> Perspective<f32> {
        self.perspective.to_perspective()
    }
    #[inline]
    pub fn ortho_perspective(&self) -> Matrix4<f32> {
        let p = self.to_perspective();
        ortho(p.left, p.right, p.bottom, p.top, p.near, p.far)
    }

    pub fn get_perspective(&self) -> PerspectiveFov<f32, Rad<f32>> {
        self.perspective.clone()
    }

    pub fn update(&mut self, movespeed: f32) {
        let mut transform;
        if self.movement.up {
            transform = self.up();
            self.offset_position(transform * movespeed);
        }
        if self.movement.down {
            transform = -self.up();
            self.offset_position(transform * movespeed);
        }
        if self.movement.left {
            transform = -self.right();
            self.offset_position(transform * movespeed);
        }
        if self.movement.right {
            transform = self.right();
            self.offset_position(transform * movespeed);
        }
        if self.movement.forward {
            transform = self.forward();
            self.offset_position(transform * movespeed);
        }
        if self.movement.backward {
            transform = -self.forward();
            self.offset_position(transform * movespeed);
        }

        let rad = Rad { s: 0.5 * f32::consts::PI / 180.0 };
        let rad_0 = Rad { s: 0.0 };
        if self.rotation.pitch_u {
            self.offset_orientation(rad, rad_0)
        }
        if self.rotation.pitch_d {
            self.offset_orientation(-rad, rad_0)
        }
        if self.rotation.yaw_l {
            self.offset_orientation(rad_0, -rad)
        }
        if self.rotation.yaw_r {
            self.offset_orientation(rad_0, rad)
        }
    }
    pub fn position_and_direction(&mut self) {
        println!("Current position at x: {}, y: {}, z: {}",
                 self.position.x,
                 self.position.y,
                 self.position.z);
    }

    pub fn process_input(&mut self, event: &glutin::Event) {
        match event {
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::E)) => {
                self.movement.up = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::E)) => {
                self.movement.up = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Q)) => {
                self.movement.down = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Q)) => {
                self.movement.down = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::A)) => {
                self.movement.left = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::A)) => {
                self.movement.left = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::D)) => {
                self.movement.right = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::D)) => {
                self.movement.right = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::W)) => {
                self.movement.forward = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::W)) => {
                self.movement.forward = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::S)) => {
                self.movement.backward = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::S)) => {
                self.movement.backward = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Up)) => {
                self.rotation.pitch_u = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Up)) => {
                self.rotation.pitch_u = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Down)) => {
                self.rotation.pitch_d = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Down)) => {
                self.rotation.pitch_d = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Left)) => {
                self.rotation.yaw_l = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Left)) => {
                self.rotation.yaw_l = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Right)) => {
                self.rotation.yaw_r = true;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Released,
                                          _,
                                          Some(glutin::VirtualKeyCode::Right)) => {
                self.rotation.yaw_r = false;
            }
            &glutin::Event::KeyboardInput(glutin::ElementState::Pressed,
                                          _,
                                          Some(glutin::VirtualKeyCode::Space)) => {
                self.vertical_angle = Rad { s: 0.0 };
                self.horizontal_angle = Rad { s: 0.0 };;
            }
            _ => {}
        }
    }
}

#[derive(Copy, Clone)]
pub struct Camera2 {
    position: Vector3<f32>,
    pub perspective: PerspectiveFov<f32, Rad<f32>>,
    vertical_angle: Rad<f32>,
    horizontal_angle: Rad<f32>,
    pub movespeed: f32,
    pub turnspeed: f32,
}

impl Camera2 {
    pub fn new(ms: f32, ts: f32) -> Camera2 {
        Camera2 {
            position: Vector3::new(0.0, 0.0, 0.0),
            perspective: PerspectiveFov {
                fovy: Rad { s: 0.69 },
                aspect: 16.0 / 9.0,
                near: 0.1,
                far: 2000.0,
            },
            vertical_angle: Rad { s: 0.0 },
            horizontal_angle: Rad { s: 0.0 },
            movespeed: ms,
            turnspeed: ts,
        }
    }

    pub fn set_position(&mut self, position: (f32, f32, f32)) {
        self.position = Vector3::from(position);
    }

    pub fn set_fov(&mut self, fov: Rad<f32>) {
        self.perspective.fovy = fov;
    }

    pub fn offset_position(&mut self, offset: Vector3<f32>) {
        self.position = self.position + (offset * self.movespeed);
    }

    pub fn offset_orientation(&mut self, up_angle: Rad<f32>, right_angle: Rad<f32>) {
        self.vertical_angle = self.vertical_angle - up_angle;
        self.horizontal_angle = self.horizontal_angle + right_angle;
    }
    pub fn update_movespeed(&mut self, u: f32) {
        self.movespeed += u;
    }

    pub fn reset_position(&mut self) {
        self.position = Vector3::new(0.0, 0.0, 0.0);
    }

    pub fn reset_orientation(&mut self) {
        self.vertical_angle = Rad { s: 0.0 };
        self.horizontal_angle = Rad { s: 0.0 };;
    }

    pub fn orientation(&self) -> Matrix3<f32> {
        let q_vertical = Quaternion::from_axis_angle(Vector3::unit_x(), self.vertical_angle);
        let q_horizontal = Quaternion::from_axis_angle(Vector3::unit_y(), self.horizontal_angle);
        Matrix3::from(&q_vertical * &q_horizontal)
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        &self.perspective() * &self.view()
    }

    pub fn forward(&self) -> Vector3<f32> {
        &self.orientation().transpose() * &Vector3::new(0.0, 0.0, -1.0)
    }

    pub fn right(&self) -> Vector3<f32> {
        &self.orientation().transpose() * &Vector3::new(1.0, 0.0, 0.0)
    }

    pub fn up(&self) -> Vector3<f32> {
        &self.orientation().transpose() * &Vector3::new(0.0, 1.0, 0.0)
    }

    pub fn view(&self) -> Matrix4<f32> {
        &Matrix4::from(self.orientation()) * &Matrix4::from_translation(-self.position)
    }

    pub fn perspective(&self) -> Matrix4<f32> {
        Matrix4::from(self.perspective)
    }

    pub fn to_perspective(&self) -> Perspective<f32> {
        self.perspective.to_perspective()
    }
    #[inline]
    pub fn ortho_perspective(&self) -> Matrix4<f32> {
        let p = self.to_perspective();
        ortho(p.left, p.right, p.bottom, p.top, p.near, p.far)
    }

    pub fn get_perspective(&self) -> PerspectiveFov<f32, Rad<f32>> {
        self.perspective.clone()
    }

    pub fn position_and_direction(&mut self) {
        println!("Current position at x: {}, y: {}, z: {}",
                 self.position.x,
                 self.position.y,
                 self.position.z);
    }
}
