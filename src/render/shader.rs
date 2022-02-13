use std::fs::File;
use std::io::{BufReader, BufRead};

pub enum ShaderBind {
    Default,
    Use(Shader)
}

pub struct ShaderProgramSource {
    pub vertex_shader: String,
    pub fragment_shader: String
}

impl ShaderProgramSource {
    pub fn empty() -> Self {
        ShaderProgramSource {
            vertex_shader: String::from(""),
            fragment_shader: String::from("")
        }
    }
}

pub struct Shader {
    pub program: glium::Program
}

impl Shader {
    pub fn compile(display: &glium::Display, path: &str) -> Self {
        let source = Shader::parse_shader(path);

        let program = glium::Program::from_source(display,
            source.vertex_shader.as_str(), source.fragment_shader.as_str(),
            None).unwrap();
        
        Shader { 
            program: program
        }
    }

    fn parse_shader(path: &str) -> ShaderProgramSource {
        // Todo: load shaders from file
        let shader_file = File::open(path).expect("Shader file could not be opened");
        let buffered = BufReader::new(shader_file);

        enum ShaderType { Vertex, Fragment }

        let mut shader_program_source = ShaderProgramSource::empty();
        
        let mut current_shader = ShaderType::Vertex;
        for line in buffered.lines() {
            let temp = line.unwrap();
            let line_str = temp.as_str();
            match line_str {
                "#shader vertex" => current_shader = ShaderType::Vertex,
                "#shader fragment" => current_shader = ShaderType::Fragment,
                sl => {
                    match current_shader {
                        ShaderType::Vertex => shader_program_source.vertex_shader.push_str(format!("{}\n", sl).as_str()),
                        ShaderType::Fragment => shader_program_source.fragment_shader.push_str(format!("{}\n", sl).as_str())
                    }
                }
            }
        }

        shader_program_source
    }
}
