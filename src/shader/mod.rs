use std::collections::HashMap;
use glium;

mod cooktorrance;

pub struct Program {
    pub none: glium::Program,
    pub ambient: glium::Program,
    pub diffuse: glium::Program,
    pub ambient_diffuse:glium::Program,
    pub diffuse_bump:glium::Program,
    pub ambient_diffuse_bump:glium::Program,
}


pub fn program_map(display: &glium::Display) -> HashMap<String, Program> {
    let mut program_map = HashMap::new();
    let name = String::from("cooktorrance");
    println!("Creating {}", &name);

    let program = Program {
        none: glium::Program::from_source(display, cooktorrance::CT_VERT, cooktorrance::CT_FRAG_PBR, None).unwrap(),
        ambient: glium::Program::from_source(display, cooktorrance::CT_VERT, cooktorrance::CT_FRAG_PBR, None).unwrap(),
        diffuse: glium::Program::from_source(display, cooktorrance::CT_VERT, cooktorrance::CT_FRAG_PBR, None).unwrap(),
        ambient_diffuse: glium::Program::from_source(display, cooktorrance::CT_VERT, cooktorrance::CT_FRAG_PBR, None).unwrap(),
        diffuse_bump: glium::Program::from_source(display, cooktorrance::CT_VERT, cooktorrance::CT_FRAG_PBR, None).unwrap(),
        ambient_diffuse_bump: glium::Program::from_source(display, cooktorrance::CT_VERT, cooktorrance::CT_FRAG_PBR, None).unwrap(),
        //ambient_diffuse_bump: glium::Program::from_source(display, tex::TEX_VERT, tex::AMBIENT_DIFFUSE_BUMP_FRAG, None).unwrap(),
    };
    program_map.insert(name, program);
    program_map
}
