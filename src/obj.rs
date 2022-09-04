use std::sync::Arc;

use crate::{
    color::Color,
    material::{EmissiveMaterial, Glass, LambertianMaterial, Material},
    primitive::{Primitive, ShapePrimitive},
    shape::Triangle,
    vector::Vector,
};

pub fn load_obj(file_name: &str, fallback_material: Arc<dyn Material>) -> Vec<Arc<dyn Primitive>> {
    let (models, input_materials) =
        tobj::load_obj(file_name, &tobj::GPU_LOAD_OPTIONS).expect("Error loading models");
    let input_materials = input_materials.expect("Error loading materials");

    let mut materials: Vec<Arc<dyn Material>> = vec![];
    for m in input_materials {
        let diffuse = Color {
            r: m.diffuse[0] as f64,
            g: m.diffuse[1] as f64,
            b: m.diffuse[2] as f64,
        };
        let ambient = if let Some(emission) = m.unknown_param.get("Ke") {
            let emission: Vec<f64> = emission.split(' ').map(|s| s.parse().unwrap()).collect();
            Some(Color {
                r: emission[0] as f64,
                g: emission[1] as f64,
                b: emission[2] as f64,
            })
        } else {
            None
        };

        if ambient.is_some() && !ambient.unwrap().is_black() {
            materials.push(Arc::new(EmissiveMaterial {
                emittance: ambient.unwrap(),
            }));
        } else {
            materials.push(Arc::new(LambertianMaterial {
                reflectance: diffuse,
            }));
        }
    }

    let mut primitives: Vec<Arc<dyn Primitive>> = Vec::new();
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;

        let material = Arc::clone(if let Some(material_id) = mesh.material_id {
            &materials[material_id]
        } else {
            &fallback_material
        });

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
                    Triangle::with_normals(
                        vertices[*i as usize],
                        vertices[*j as usize],
                        vertices[*k as usize],
                        normals[*i as usize],
                        normals[*j as usize],
                        normals[*k as usize],
                    )
                } else {
                    Triangle::new(
                        vertices[*i as usize],
                        vertices[*j as usize],
                        vertices[*k as usize],
                    )
                };
                primitives.push(Arc::new(ShapePrimitive {
                    shape: Box::new(triangle),
                    material: Arc::clone(&material),
                }));
            }
        }
    }
    primitives
}