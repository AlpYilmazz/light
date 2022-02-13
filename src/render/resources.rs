
use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct Vertex2D {
    pub position: [f32; 2],
}

impl Vertex2D {
    pub fn new(pos: [f32; 2]) -> Self {
        Vertex2D {
            position: pos
        }
    }
}

implement_vertex!(Vertex2D, position);

pub struct VertexBuffer<V: glium::Vertex> {
    buffer: Vec<V>
}

impl<V> VertexBuffer<V>
where
    V: glium::Vertex
{
    pub fn new(buf: Vec<V>) -> Self {
        VertexBuffer {
            buffer: buf
        }
    }

    pub fn borrow(&self) -> &[V] {
        &self.buffer
    }
}

pub struct IndexBuffer {
    buffer: Vec<u32>
}

impl IndexBuffer {
    pub fn new(buf: Vec<u32>) -> Self {
        IndexBuffer {
            buffer: buf
        }
    }

    pub fn borrow(&self) -> &[u32] {
        &self.buffer
    }
}