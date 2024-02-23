use crate::wgpu_winit::run;

mod canvas;
mod wgpu_winit;
mod texture;

fn main() {
    println!("Hello, world!");
    pollster::block_on(run());
}
