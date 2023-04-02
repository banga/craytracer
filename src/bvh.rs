use std::{fmt::Display, sync::Arc};

use crate::{
    bounds::Bounds,
    intersection::PrimitiveIntersection,
    primitive::Primitive,
    ray::Ray,
    vector::{Axis, AXES},
};

pub struct Split {
    axis: Axis,
    location: f64,
}

pub struct PrimitiveInfo {
    primitive: Arc<dyn Primitive>,
    bounds: Bounds,
}

pub enum BvhNode {
    InteriorNode {
        bounds: Bounds,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
        split: Split,
    },
    LeafNode {
        bounds: Bounds,
        primitive_infos: Vec<PrimitiveInfo>,
    },
}

impl Display for PrimitiveInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.bounds))
    }
}

impl BvhNode {
    pub fn new(primitives: Vec<Arc<dyn Primitive>>) -> Box<BvhNode> {
        let primitive_infos = primitives
            .iter()
            .map(|p| PrimitiveInfo {
                primitive: Arc::clone(p),
                bounds: p.bounds(),
            })
            .collect();

        BvhNode::from_primitive_infos(primitive_infos)
    }

    fn from_primitive_infos(primitive_infos: Vec<PrimitiveInfo>) -> Box<BvhNode> {
        assert!(primitive_infos.len() > 0);
        let bounds: Bounds = primitive_infos.iter().map(|p| p.bounds).sum();

        if primitive_infos.len() <= 4 {
            return Box::new(BvhNode::LeafNode {
                bounds,
                primitive_infos,
            });
        }

        let split = BvhNode::find_split(&primitive_infos);
        let (left, right): (Vec<PrimitiveInfo>, Vec<PrimitiveInfo>) = primitive_infos
            .into_iter()
            .partition(|primitive| primitive.bounds.max[split.axis] <= split.location);

        if left.len() == 0 {
            Box::new(BvhNode::LeafNode {
                bounds,
                primitive_infos: right,
            })
        } else if right.len() == 0 {
            Box::new(BvhNode::LeafNode {
                bounds,
                primitive_infos: left,
            })
        } else {
            Box::new(BvhNode::InteriorNode {
                bounds,
                left: BvhNode::from_primitive_infos(left),
                right: BvhNode::from_primitive_infos(right),
                split,
            })
        }
    }

    fn find_split(primitives: &Vec<PrimitiveInfo>) -> Split {
        // TODO: This is a very naive implementation that just always picks the
        // median edge. We should use a SAH to find a better split.
        let extents = primitives
            .iter()
            .map(|p| p.bounds)
            .reduce(|x, y| x + y)
            .unwrap();

        let span = extents.span();
        let max_axis = AXES
            .iter()
            .max_by(|&x, &y| span[*x].total_cmp(&span[*y]))
            .unwrap();

        let mut locations: Vec<f64> = primitives.iter().map(|p| p.bounds.max[*max_axis]).collect();
        locations.sort_by(|a, b| a.total_cmp(b));

        let location = locations[(locations.len() - 1) / 2];

        Split {
            axis: *max_axis,
            location,
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        let bounds = match self {
            Self::LeafNode {
                bounds,
                primitive_infos: _,
            } => bounds,
            Self::InteriorNode {
                bounds,
                left: _,
                right: _,
                split: _,
            } => bounds,
        };

        // If the ray doesn't intersect the bounds, it could still be contained
        // within them, so check for both
        if bounds.intersect(ray).is_none() && !bounds.contains(&ray.origin) {
            return None;
        }

        match self {
            Self::LeafNode {
                bounds: _,
                primitive_infos,
            } => {
                let mut best_intersection: Option<PrimitiveIntersection> = None;
                for primitive_info in primitive_infos {
                    if let Some(intersection) = primitive_info.primitive.intersect(ray) {
                        if best_intersection.is_none()
                            || intersection.distance < best_intersection.as_ref().unwrap().distance
                        {
                            best_intersection = Some(intersection);
                        }
                    }
                }
                best_intersection
            }
            Self::InteriorNode {
                bounds: _,
                left,
                right,
                split,
            } => {
                let (x, y) = if ray.direction[split.axis] < 0.0 {
                    (right.intersect(ray), left.intersect(ray))
                } else {
                    (left.intersect(ray), right.intersect(ray))
                };

                return if x.is_none() {
                    y
                } else if y.is_none() {
                    x
                } else if x.as_ref().unwrap().distance < y.as_ref().unwrap().distance {
                    x
                } else {
                    y
                };
            }
        }
    }
}

impl Display for BvhNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeafNode {
                bounds,
                primitive_infos,
            } => f.write_str(&format!(
                "primitives: ({:?}) bounds: ({:?})",
                primitive_infos
                    .iter()
                    .map(|p| format!("{:?} ", p.bounds))
                    .collect::<String>(),
                bounds,
            )),
            Self::InteriorNode {
                left,
                right,
                split,
                bounds,
            } => f.write_str(&format!(
                "left: ({}), right: ({}), split: ({:?}, {}), bounds: {:?}",
                left, right, split.axis, split.location, bounds
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::BvhNode;
    use crate::{
        color::Color,
        material::{Material, MatteMaterial},
        primitive::ShapePrimitive,
        ray::Ray,
        shape::Sphere,
        vector::Vector,
    };

    #[test]
    fn bvh_node() {
        let material: Arc<dyn Material> = Arc::new(MatteMaterial::new(Color::WHITE, 0.0));
        let node = BvhNode::new(vec![
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere::new(Vector(0.5, 0.5, 0.5), 0.5)),
                material: Arc::clone(&material),
            }),
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere::new(Vector(1.5, 0.5, 0.5), 0.5)),
                material: Arc::clone(&material),
            }),
        ]);

        // Intersect from left
        assert_eq!(
            Vector(0.0, 0.5, 0.5),
            node.intersect(&mut Ray::new(Vector(-1.0, 0.5, 0.5), Vector::X,))
                .unwrap()
                .location
        );

        // Intersect from right
        assert_eq!(
            Vector(2.0, 0.5, 0.5),
            node.intersect(&mut Ray::new(Vector(3.0, 0.5, 0.5), -Vector::X,))
                .unwrap()
                .location
        );

        // Intersect from inside first sphere
        assert_eq!(
            Vector(1.0, 0.5, 0.5),
            node.intersect(&mut Ray::new(Vector(0.5, 0.5, 0.5), Vector::X,))
                .unwrap()
                .location
        );
        assert_eq!(
            Vector(0.0, 0.5, 0.5),
            node.intersect(&mut Ray::new(Vector(0.5, 0.5, 0.5), -Vector::X,))
                .unwrap()
                .location
        );
    }
}
