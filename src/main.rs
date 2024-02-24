use crate::wgpu_winit::run;

mod canvas;
mod wgpu_winit;
mod texture;
mod tessellate;

fn main() {
    println!("Hello, world!");
    let mut line = canvas::Line::start(0., 0.);
    line.to(1., 1.);
    line.to(2., 4.3);
    line.end(&mut canvas::Canvas {});
    pollster::block_on(run());
}
