use std::sync::Arc;

use crate::{
    color::Color,
    material::{EmissiveMaterial, LambertianMaterial, Material},
    primitive::{Primitive, ShapePrimitive},
    shape::Triangle,
    vector::Vector,
};

pub fn load_obj(file_name: &str) -> Vec<Box<dyn Primitive>> {
    let (models, input_materials) =
        tobj::load_obj(file_name, &tobj::GPU_LOAD_OPTIONS).expect("Error loading models");
    let input_materials = input_materials.expect("Error loading materials");

    let mut materials: Vec<Arc<dyn Material>> = vec![];
    for m in input_materials {
        if let Some(emission) = m.unknown_param.get("Ke") {
            let emission: Vec<f64> = emission.split(' ').map(|s| s.parse().unwrap()).collect();
            let ambient = Color {
                r: emission[0] as f64,
                g: emission[1] as f64,
                b: emission[2] as f64,
            };
            materials.push(Arc::new(EmissiveMaterial { emittance: ambient }));
        } else {
            let diffuse = Color {
                r: m.diffuse[0] as f64,
                g: m.diffuse[1] as f64,
                b: m.diffuse[2] as f64,
            };
            materials.push(Arc::new(LambertianMaterial {
                reflectance: diffuse,
            }));
        }
    }

    let fallback_material: Arc<dyn Material> = Arc::new(LambertianMaterial {
        reflectance: Color::WHITE,
    });
    let mut primitives: Vec<Box<dyn Primitive>> = Vec::new();
    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;

        let positions = &mesh.positions;
        let indices = &mesh.indices;
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
        for i in (0..mesh.positions.len()).step_by(3) {
            vertices.push(Vector(
                positions[i] as f64,
                positions[i + 1] as f64,
                positions[i + 2] as f64,
            ));
        }

        for i in (0..indices.len()).step_by(3) {
            let triangle = Triangle::new(
                vertices[indices[i] as usize],
                vertices[indices[i + 1] as usize],
                vertices[indices[i + 2] as usize],
            );
            primitives.push(Box::new(ShapePrimitive {
                shape: Box::new(triangle),
                material: Arc::clone(&material),
            }));
        }
    }
    primitives
}
