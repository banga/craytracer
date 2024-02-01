use std::{collections::HashMap, f64::consts::E, path::Path, sync::Arc};

use image::DynamicImage;
use log::{debug, warn};

use crate::{
    color::Color,
    geometry::{point::Point, vector::Vector},
    light::Light,
    material::Material,
    primitive::Primitive,
    shape::Shape,
    texture::Texture,
};

fn load_texture(file_name: &str, texture_file_name: &str) -> DynamicImage {
    let path = Path::new(file_name)
        .parent()
        .unwrap_or(Path::new(""))
        .join(texture_file_name);
    debug!("Loading texture from {:?}", path);
    image::io::Reader::open(&path).expect(
        &format!("Could not find texture file \"{}\"", texture_file_name)).decode().unwrap()
}

pub fn load_obj(file_name: &str, fallback_material: Arc<Material>) -> Vec<Arc<Primitive>> {
    debug!("Loading mesh from \"{}\"", file_name);

    let (models, input_materials) = tobj::load_obj(file_name, &tobj::GPU_LOAD_OPTIONS).unwrap();
    let input_materials = match input_materials {
        Ok(m) => m,
        Err(e) => {
            warn!(
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
            warn!("Could not parse float from '{}'", s);
            0.0
        }
    }

    fn parse_float_3(s: &str) -> [f64; 3] {
        let mut iter = s.split_whitespace().map(parse_float);
        [
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ]
    }

    let mut materials = Vec::new();
    let mut emittances = HashMap::new();
    for (id, m) in input_materials.iter().enumerate() {
        debug!("Creating material \"{}\":", m.name);

        let diffuse: Texture<Color> = match &m.diffuse_texture {
            None => Texture::constant(m.diffuse.unwrap().into()),
            Some(texture_file_name) => Texture::image(load_texture(file_name, texture_file_name)),
        };

        let specular: Texture<Color> = match &m.specular_texture {
            None => Texture::constant(m.specular.map(|c| c.into()).unwrap_or(Color::BLACK)),
            Some(texture_file_name) => Texture::image(load_texture(file_name, texture_file_name)),
        };

        let emittance: Color = m
            .unknown_param
            .get("Ke")
            .map(|emission| parse_float_3(emission).into())
            .unwrap_or(Color::BLACK);

        let shininess: f64 = m.shininess.unwrap_or(0.0);
        // TODO: Figure out how to properly convert these
        let roughness = Texture::constant(180.0 * (1.0 - E.powf(-shininess / 100.0)));

        let dissolve: f64 = m.dissolve.unwrap_or(1.0);

        let material = if !emittance.is_black() {
            emittances.insert(id, emittance);
            Arc::clone(&fallback_material)
        } else if dissolve < 1.0 {
            // TODO: Use "dissolve"?
            let eta = m.optical_density.unwrap_or(1.0);
            Arc::new(Material::new_glass(diffuse.clone(), diffuse.clone(), eta))
        } else {
            // This is a hacky way to support reflective surfaces. We should
            // likely switch to glTF or something
            match m.illumination_model {
                Some(3 | 4 | 5 | 6 | 7 | 8 | 9) => Arc::new(Material::new_metal(diffuse, specular)),
                _ => Arc::new(Material::new_plastic(diffuse, specular, roughness)),
            }
        };
        debug!("\t{:?}", material);
        materials.push(material);
    }

    let mut primitives: Vec<Arc<Primitive>> = Vec::new();
    for (i, model) in models.iter().enumerate() {
        debug!("Loading model \"{}\":", model.name,);

        let mesh = &model.mesh;

        let (material, emittance) = if let Some(material_id) = mesh.material_id {
            (&materials[material_id], emittances.get(&material_id))
        } else {
            (&fallback_material, None)
        };

        assert!(
            mesh.positions.len() % 3 == 0,
            "all faces should be triangles in model #{}: '{}'",
            i,
            model.name
        );

        let vertices: Vec<Point> = mesh
            .positions
            .chunks_exact(3)
            .map(|p| {
                // Convert from right-handed to left-handed coordinate system
                Point(p[0], p[1], -p[2])
            })
            .collect();

        let normals: Vec<Vector> = mesh
            .normals
            .chunks_exact(3)
            .map(|n| {
                // Convert from right-handed to left-handed coordinate system
                Vector(n[0], n[1], -n[2])
            })
            .collect();

        let texture_coordinates: Vec<(f64, f64)> = mesh
            .texcoords
            .chunks_exact(2)
            .map(|tc| 
                // Convert from right-handed to left-handed coordinate system
                (tc[0], 1.0 - tc[1])
            )
            .collect();

        for chunk in mesh.indices.chunks(3) {
            if let &[i, j, k] = chunk {
                let vi = vertices[i as usize];
                let vj = vertices[j as usize];
                let vk = vertices[k as usize];

                let normal = (vk - vi).cross(&(vj - vi)).normalized();
                let mut ni = normal;
                let mut nj = normal;
                let mut nk = normal;
                if normals.len() > 0 {
                    ni = normals[i as usize];
                    nj = normals[j as usize];
                    nk = normals[k as usize];
                }

                let mut uv0 = (0.0, 0.0);
                let mut uv1 = (1.0, 0.0);
                let mut uv2 = (1.0, 1.0);
                if texture_coordinates.len() > 0 {
                    uv0 = texture_coordinates[i as usize];
                    uv1 = texture_coordinates[j as usize];
                    uv2 = texture_coordinates[k as usize];
                }

                let triangle = Shape::new_triangle_with_normals_and_texture_coordinates(
                    vi, vj, vk, ni, nj, nk, uv0, uv1, uv2,
                );

                if let Some(triangle) = triangle {
                    let triangle = Arc::new(triangle);
                    let primitive = match emittance {
                        None => Primitive::new(Arc::clone(&triangle), Arc::clone(&material)),
                        Some(&emittance) => Primitive::new_area_light(
                            Arc::clone(&triangle),
                            Arc::new(Light::Area {
                                shape: Arc::clone(&triangle),
                                emittance,
                            }),
                        ),
                    };
                    primitives.push(Arc::new(primitive));
                } else {
                    debug!("\tSkipping degenerate triangle with vertices {i}, {j}, {k}");
                }
            }
        }

        debug!(
            "\t\"{}\" material\n\t{} triangles\n\t{} vertices\n\t{} normals",
            match mesh.material_id {
                Some(id) => &input_materials[id].name,
                None => "fallback",
            },
            mesh.indices.len() / 3,
            mesh.positions.len() / 3,
            mesh.normals.len() / 3,
        );
    }

    debug!(
        "Loaded {} primitives from {} mesh",
        primitives.len(),
        file_name
    );

    primitives
}
