use time;

pub mod math;
pub mod graphics;


//Adapted from glium tutorial

pub enum Action {
    Stop,
    Continue,
}

#[inline]
pub fn start_loop<F>(mut callback: F)
    where F: FnMut() -> Action
{
    use std::time::{Duration};
    loop {
        let now = time::PreciseTime::now();
        match callback() {
            Action::Stop => break,
            Action::Continue => (),
        };
        while now.to(time::PreciseTime::now()) < time::Duration::milliseconds(7) {

        }
    }
}

pub fn timer<F>(mut callback: F)
    where F: FnMut()
{
    use time::PreciseTime;
    let start = time::PreciseTime::now();
    callback();
    let stop = time::PreciseTime::now();

    let time = start.to(stop);
    println!("{}ms elapsed", time.num_milliseconds());
}
