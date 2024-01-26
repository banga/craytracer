use std::env::args;

use craytracer::{bounds::Bounds, p};
use tobj::{Material, Mesh};

fn analyze_obj(file_name: &str) {
    println!("Loading mesh from \"{}\"", file_name);

    let (models, materials) =
        tobj::load_obj(file_name, &tobj::GPU_LOAD_OPTIONS).expect("Error loading models");
    let materials = materials.expect("Missing materials");

    for m in &materials {
        println!("Material: \"{}\"", m.name);
        let Material {
            name: _,
            ambient,
            diffuse,
            specular,
            shininess,
            dissolve,
            optical_density,
            ambient_texture,
            diffuse_texture,
            specular_texture,
            normal_texture,
            shininess_texture,
            dissolve_texture,
            illumination_model,
            unknown_param,
        } = m;
        if let Some(ambient) = ambient {
            println!("\tambient = {:?}", ambient);
        }
        if let Some(diffuse) = diffuse {
            println!("\tdiffuse = {:?}", diffuse);
        }
        if let Some(specular) = specular {
            println!("\tspecular = {:?}", specular);
        }
        if let Some(shininess) = shininess {
            println!("\tshininess = {:?}", shininess);
        }
        if let Some(dissolve) = dissolve {
            println!("\tdissolve = {:?}", dissolve);
        }
        if let Some(optical_density) = optical_density {
            println!("\toptical_density = {:?}", optical_density);
        }
        if let Some(ambient_texture) = ambient_texture {
            println!("\tambient_texture = {:?}", ambient_texture);
        }
        if let Some(diffuse_texture) = diffuse_texture {
            println!("\tdiffuse_texture = {:?}", diffuse_texture);
        }
        if let Some(specular_texture) = specular_texture {
            println!("\tspecular_texture = {:?}", specular_texture);
        }
        if let Some(normal_texture) = normal_texture {
            println!("\tnormal_texture = {:?}", normal_texture);
        }
        if let Some(shininess_texture) = shininess_texture {
            println!("\tshininess_texture = {:?}", shininess_texture);
        }
        if let Some(dissolve_texture) = dissolve_texture {
            println!("\tdissolve_texture = {:?}", dissolve_texture);
        }
        if let Some(illumination_model) = illumination_model {
            println!("\tillumination_model = {:?}", illumination_model);
        }
        for (key, value) in unknown_param {
            println!("\tunknown_param \"{}\" = \"{}\"", key, value);
        }
        println!();
    }

    let mut world_bounds: Option<Bounds> = None;
    for model in models.iter() {
        println!("Model: {}", model.name);
        let Mesh {
            material_id,
            positions,
            vertex_color,
            normals,
            texcoords,
            indices,
            face_arities,
            texcoord_indices,
            normal_indices,
        } = &model.mesh;
        if let Some(material_id) = material_id {
            println!("\tmaterial: \"{}\"", materials[*material_id].name);
        }
        if positions.len() > 0 {
            println!("\t{} positions", positions.len());
            let bounds: Bounds = positions
                .chunks_exact(3)
                .map(|p| Bounds::new(p!(p[0], p[1], p[2]), p!(p[0], p[1], p[2])))
                .sum();
            println!("\t\tmin: {}", bounds.min);
            println!("\t\tmax: {}", bounds.max);
            world_bounds = world_bounds.or(Some(bounds)).map(|b| b + bounds);
        }
        if vertex_color.len() > 0 {
            println!("\t{} vertex_color", vertex_color.len());
        }
        if normals.len() > 0 {
            println!("\t{} normals", normals.len());
        }
        if texcoords.len() > 0 {
            println!("\t{} texcoords", texcoords.len());
        }
        if indices.len() > 0 {
            println!("\t{} indices", indices.len());
        }
        if face_arities.len() > 0 {
            println!("\t{} face_arities", face_arities.len());
        }
        if texcoord_indices.len() > 0 {
            println!("\t{} texcoord_indices", texcoord_indices.len());
        }
        if normal_indices.len() > 0 {
            println!("\t{} normal_indices", normal_indices.len());
        }
        println!();
    }

    println!("\nWorld:");
    println!("\t{} models", models.len());
    println!("\t{} materials", materials.len());
    if let Some(world_bounds) = world_bounds {
        println!("\tBounds: {} -> {}", world_bounds.min, world_bounds.max);
    }
}

fn main() {
    let obj_file_name = args().nth(1).expect("Missing arg for file name");
    analyze_obj(&obj_file_name);
}
