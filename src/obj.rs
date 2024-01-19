use std::sync::Arc;

use crate::{
    color::Color,
    geometry::{point::Point, vector::Vector},
    material::Material,
    primitive::Primitive,
    shape::Shape,
};

pub fn load_obj(file_name: &str, fallback_material: Arc<Material>) -> Vec<Arc<Primitive>> {
    println!("Loading mesh from \"{}\"", file_name);

    let (models, input_materials) = tobj::load_obj(file_name, &tobj::GPU_LOAD_OPTIONS).unwrap();
    let input_materials = match input_materials {
        Ok(m) => m,
        Err(e) => {
            println!(
                "Error loading materials in {:?}: {}, skipping",
                file_name, e
            );
            vec![]
        }
    };

    fn parse_float(s: &str) -> f64 {
        if let Ok(f) = s.parse::<f64>() {
            f
        } else if let Ok(i) = s.parse::<i32>() {
            i as f64
        } else {
            println!("Could not parse float from '{}'", s);
            0.0
        }
    }

    let mut materials: Vec<Arc<Material>> = vec![];
    for m in &input_materials {
        let diffuse = if let Some([r, g, b]) = m.diffuse {
            Color {
                r: r as f64,
                g: g as f64,
                b: b as f64,
            }
        } else {
            Color::WHITE
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
            // TODO: add area lights
            materials.push(Arc::clone(&fallback_material));
        } else if !diffuse.is_black() {
            // Hack: ignore completely black materials because we don't render
            // them correctly yet. These tend to be image textured materials.
            // TODO: read roughness
            materials.push(Arc::new(Material::new_matte(diffuse, 0.0)));
        } else {
            materials.push(Arc::clone(&fallback_material));
        }
    }

    let mut primitives: Vec<Arc<Primitive>> = Vec::new();
    for (i, model) in models.iter().enumerate() {
        let mesh = &model.mesh;

        let material = if let Some(material_id) = mesh.material_id {
            &materials[material_id]
        } else {
            &fallback_material
        };

        assert!(
            mesh.positions.len() % 3 == 0,
            "all faces should be triangles in model #{}: '{}'",
            i,
            model.name
        );

        let mut vertices = Vec::new();
        for chunk in mesh.positions.chunks(3) {
            if let [x, y, z] = chunk {
                vertices.push(Point(
                    *x as f64, *y as f64,
                    // Convert from right-handed to left-handed coordinate system
                    -*z as f64,
                ));
            }
        }

        let mut normals = Vec::new();
        for chunk in mesh.normals.chunks(3) {
            if let [x, y, z] = chunk {
                normals.push(Vector(
                    *x as f64, *y as f64,
                    // Convert from right-handed to left-handed coordinate system
                    -*z as f64,
                ));
            }
        }

        for chunk in mesh.indices.chunks(3) {
            if let [i, j, k] = chunk {
                let vi = vertices[*i as usize];
                let vj = vertices[*j as usize];
                let vk = vertices[*k as usize];

                let triangle = if normals.len() > 0 {
                    let ni = normals[*i as usize];
                    let nj = normals[*j as usize];
                    let nk = normals[*k as usize];
                    Shape::new_triangle_with_normals(vi, vj, vk, ni, nj, nk)
                } else {
                    Shape::new_triangle(vi, vj, vk)
                };

                if let Some(triangle) = triangle {
                    primitives.push(Arc::new(Primitive::new(
                        Arc::new(triangle),
                        Arc::clone(material),
                        None,
                    )));
                } else {
                    println!("Skipping degenerate triangle with vertices {i}, {j}, {k}");
                }
            }
        }

        println!(
            "Loaded mesh \"{}\" with {} material, {} triangles, {} vertices and {} normals {:?}",
            model.name,
            match mesh.material_id {
                Some(id) => format!("\"{}\"", &input_materials[id].name),
                None => "fallback".to_string(),
            },
            mesh.indices.len() / 3,
            mesh.positions.len() / 3,
            mesh.normals.len() / 3,
            material,
        );
    }

    println!(
        "Loaded {} primitives from {} mesh",
        primitives.len(),
        file_name
    );

    primitives
}
