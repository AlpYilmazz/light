
use light::log;
use light::render::renderer::Renderer;
use light::render::shader::Shader;
use light::{error, warn, info, debug, trace};

use glium::glutin;

fn main() {
    log::init();
    info!("Starting...");

    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let shader = Shader::compile(&display, "res/basic.shader");
    let mut renderer = Renderer::new(display);
    renderer.bind_shader(shader);
    
    event_loop.run(move |ev, _, control_flow| {
        // println!("event: {:?}", ev);
        
        let next_frame_time = std::time::Instant::now() +
        std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        
        match ev {
            glutin::event::Event::MainEventsCleared => {
                renderer.begin_frame();
                renderer.clear_color(0.0, 0.0, 1.0, 1.0);
                renderer.end_frame().unwrap();
            }
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => ()
            },
            _ => ()
        }
    });
}
