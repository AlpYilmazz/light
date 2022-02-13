
use glium::Surface;

use super::{shader::Shader, resources::{IndexBuffer, VertexBuffer}};

pub enum RenderResponse {
    NoFault,
    SequenceFault
}

pub struct Renderer {
    display: glium::Display,
    shader: Option<Shader>,
    current_frame: Option<glium::Frame>
}

impl Renderer {
    pub fn new(display: glium::Display) -> Self {
        Renderer {
            display: display,
            shader: None,
            current_frame: None
        }
    }

    pub fn bind_shader(&mut self, shader: Shader) {
        self.shader = Some(shader);
    }

    pub fn begin_frame(&mut self) -> RenderResponse {
        match self.current_frame {
            Some(_) => RenderResponse::SequenceFault,
            None => {
                self.current_frame = Some(self.display.draw());
                RenderResponse::NoFault
            }
        }
    }

    pub fn end_frame(&mut self) -> Result<RenderResponse, glium::SwapBuffersError> {
        let frame = self.current_frame.take();
        match frame {
            Some(fr) => fr.finish().map(|_| RenderResponse::NoFault),
            None => Ok(RenderResponse::SequenceFault)
        }
    }

    pub fn clear_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        match &mut self.current_frame {
            Some(fr) => {
                fr.clear_color(red, green, blue, alpha);
            },
            None => ()
        };
    }

    pub fn draw<V, U>(&mut self,
        vertex_buffer: &VertexBuffer<V>,
        index_buffer: &IndexBuffer,
        uniforms: &U) -> Result<RenderResponse, glium::DrawError>
    where
        V: glium::Vertex,
        U: glium::uniforms::Uniforms
    {
        match &mut self.current_frame {
            Some(frame) => {
                let use_shader = &self.shader.as_ref();
        
                let vertex_buffer = 
                        glium::VertexBuffer::new(&self.display, vertex_buffer.borrow()).unwrap();
                // let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
                let index_buffer = glium::IndexBuffer::new(&self.display,
                    glium::index::PrimitiveType::TrianglesList,
                    index_buffer.borrow()).unwrap();
          
                frame.draw(&vertex_buffer, &index_buffer,
                    &use_shader.unwrap().program, uniforms,
                    &Default::default())
                    .map(|_| RenderResponse::NoFault)
            }
            None => Ok(RenderResponse::SequenceFault)
        }
    }
}