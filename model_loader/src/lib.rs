use std::io::Read;
use std::fs::File;
use std::io::BufReader;
use wavefront_obj::obj::*;
use cgmath::{Vector3};
use geometry::aabb::{BBox, Triangle};

pub fn load_triangles_from_obj(file_name: &'static str) -> Option<(Vec<Triangle>, BBox)> {

    let file_content = {
      let mut file = File::open(file_name).map_err(|e| format!("cannot open file: {}", e)).unwrap();
      let mut content = String::new();
      file.read_to_string(&mut content).unwrap();
      content
    };

    let obj_set = parse(file_content).map_err(|e| format!("cannot parse: {:?}", e)).unwrap();
    let objects = obj_set.objects;

    let mut aabb = BBox { min: Vector3::<f32>::new(0.0, 0.0, 0.0), max: Vector3::<f32>::new(0.0, 0.0, 0.0), };
    let mut result: Vec<Triangle> = Vec::new();

    if objects.len() == 1 {
        for shape in &objects[0].geometry[0].shapes {
            match shape.primitive {
                Primitive::Triangle((ia, _, _), (ib, _, _), (ic, _, _)) => {

                    let vertex_a = objects[0].vertices[ia];
                    let vertex_b = objects[0].vertices[ib];
                    let vertex_c = objects[0].vertices[ic];

                    let vec_a = Vector3::<f32>::new(vertex_a.x as f32, vertex_a.y as f32, vertex_a.z as f32) * 10.0; 
                    let vec_b = Vector3::<f32>::new(vertex_b.x as f32, vertex_b.y as f32, vertex_b.z as f32) * 10.0; 
                    let vec_c = Vector3::<f32>::new(vertex_c.x as f32, vertex_c.y as f32, vertex_c.z as f32) * 10.0; 

                    aabb.expand(&vec_a);
                    aabb.expand(&vec_b);
                    aabb.expand(&vec_c);

                    let tr = Triangle {
                        a: vec_a,
                        b: vec_b,
                        c: vec_c,
                    };

                    result.push(tr);
                }
                Primitive::Line(_, _) => { panic!("load_triangles_from_obj not supporting lines."); }
                Primitive::Point(_) => { panic!("load_triangles_from_obj not supporting points."); }
            }
        }
    }
    Some((result, aabb))
}
