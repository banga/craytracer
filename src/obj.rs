use std::sync::Arc;

use crate::{color::Color, material::Material, primitive::Primitive, shape::Shape, vector::Vector};

pub fn load_obj(file_name: &str, fallback_material: Arc<Material>) -> Vec<Arc<Primitive>> {
    let (models, input_materials) =
        tobj::load_obj(file_name, &tobj::GPU_LOAD_OPTIONS).expect("Error loading models");
    let input_materials = input_materials.expect("Error loading materials");

    fn parse_float(s: &str) -> f64 {
        if let Ok(f) = s.parse::<f64>() {
            f
        } else if let Ok(i) = s.parse::<i32>() {
            i as f64
        } else {
            eprintln!("Could not parse float from '{}'", s);
            0.0
        }
    }

    let mut materials: Vec<Arc<Material>> = vec![];
    for m in input_materials {
        let diffuse = Color {
            r: m.diffuse[0] as f64,
            g: m.diffuse[1] as f64,
            b: m.diffuse[2] as f64,
        };
        let ambient = if let Some(emission) = m.unknown_param.get("Ke") {
            let parts: Vec<&str> = emission.split_whitespace().collect();
            let emission: Vec<f64> = parts.iter().map(|s| parse_float(s)).collect();
            Some(Color {
                r: emission[0] as f64,
                g: emission[1] as f64,
                b: emission[2] as f64,
            })
        } else {
            None
        };

        if ambient.is_some() && !ambient.unwrap().is_black() {
            materials.push(Arc::new(Material::new_emissive(ambient.unwrap())));
        } else {
            // TODO: read roughness
            materials.push(Arc::new(Material::new_matte(diffuse, 0.0)));
        }
    }

    let mut primitives: Vec<Arc<Primitive>> = Vec::new();
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;

        let material = if let Some(material_id) = mesh.material_id {
            &materials[material_id]
        } else {
            &fallback_material
        };

        println!("Loading model {}: '{}'", i, m.name);
        assert!(
            mesh.positions.len() % 3 == 0,
            "all faces should be triangles"
        );

        let mut vertices = Vec::new();
        for chunk in mesh.positions.chunks(3) {
            if let [x, y, z] = chunk {
                vertices.push(Vector(*x as f64, *y as f64, *z as f64));
            }
        }

        let mut normals = Vec::new();
        for chunk in mesh.normals.chunks(3) {
            if let [x, y, z] = chunk {
                normals.push(Vector(*x as f64, *y as f64, *z as f64));
            }
        }

        for chunk in mesh.indices.chunks(3) {
            if let [i, j, k] = chunk {
                let triangle = if normals.len() > 0 {
                    Shape::new_triangle_with_normals(
                        vertices[*i as usize],
                        vertices[*j as usize],
                        vertices[*k as usize],
                        normals[*i as usize],
                        normals[*j as usize],
                        normals[*k as usize],
                    )
                } else {
                    Shape::new_triangle(
                        vertices[*i as usize],
                        vertices[*j as usize],
                        vertices[*k as usize],
                    )
                };
                primitives.push(Arc::new(Primitive::new_shape_primitive(
                    Arc::new(triangle),
                    Arc::clone(material),
                )));
            }
        }
    }
    primitives
}
