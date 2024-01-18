use std::{fmt::Display, sync::Arc};

use crate::{
    bounds::Bounds,
    geometry::{point::Point, Axis},
    intersection::PrimitiveIntersection,
    primitive::Primitive,
    ray::Ray,
};

#[derive(Debug, PartialEq)]
pub struct Split {
    axis: Axis,
    location: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrimitiveInfo {
    primitive: Arc<Primitive>,
    bounds: Bounds,
    centroid: Point,
}

#[derive(Debug, PartialEq)]
pub struct Bvh {
    root: BvhNode,
}

impl Bvh {
    pub fn new(primitives: Vec<Arc<Primitive>>) -> Self {
        let mut primitive_infos: Vec<_> = primitives
            .iter()
            .map(|p| PrimitiveInfo {
                primitive: Arc::clone(p),
                bounds: p.bounds(),
                centroid: p.bounds().centroid(),
            })
            .collect();

        let root = BvhNode::from_primitive_infos(&mut primitive_infos);
        Bvh { root }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<PrimitiveIntersection> {
        let mut q = Vec::with_capacity(32);
        q.push(&self.root);
        let mut current: Option<PrimitiveIntersection> = None;
        while let Some(node) = q.pop() {
            let bounds = match node {
                BvhNode::LeafNode { bounds, .. } => bounds,
                BvhNode::InteriorNode { bounds, .. } => bounds,
            };

            // If the ray doesn't intersect the bounds, it could still be contained
            // within them, so check for both
            let bounds_intersection = bounds.intersect(ray);
            if bounds_intersection.is_none() && !bounds.contains(&ray.origin) {
                continue;
            }

            match node {
                BvhNode::LeafNode {
                    primitive_infos, ..
                } => {
                    for primitive_info in primitive_infos {
                        if let Some(intersection) = primitive_info.primitive.intersect(ray) {
                            if current.is_none()
                                || intersection.distance < current.as_ref().unwrap().distance
                            {
                                current = Some(intersection);
                            }
                        }
                    }
                }
                BvhNode::InteriorNode {
                    left, right, split, ..
                } => {
                    if ray.direction[split.axis] < 0.0 {
                        q.push(left);
                        q.push(right);
                    } else {
                        q.push(right);
                        q.push(left);
                    };
                }
            }
        }

        current
    }
}

#[derive(Debug, PartialEq)]
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
    fn from_primitive_infos(mut primitive_infos: &mut [PrimitiveInfo]) -> BvhNode {
        assert!(primitive_infos.len() > 0);
        let bounds: Bounds = primitive_infos.iter().map(|p| p.bounds).sum();

        if primitive_infos.len() <= 4 {
            return BvhNode::LeafNode {
                bounds,
                primitive_infos: primitive_infos.to_vec(),
            };
        }

        let split = BvhNode::find_split(&mut primitive_infos);
        if split.is_none() {
            return BvhNode::LeafNode {
                bounds,
                primitive_infos: primitive_infos.to_vec(),
            };
        }
        let split = split.unwrap();
        let (l, r) = primitive_infos.split_at_mut(split.location);
        BvhNode::InteriorNode {
            bounds,
            left: Box::new(BvhNode::from_primitive_infos(l)),
            right: Box::new(BvhNode::from_primitive_infos(r)),
            split,
        }
    }

    fn find_split(primitives: &mut [PrimitiveInfo]) -> Option<Split> {
        // TODO: This is a very naive implementation that just always picks the
        // median edge. We should use a SAH to find a better split.
        let extents = primitives.iter().map(|p| p.centroid).fold(
            Bounds::new(primitives[0].centroid, primitives[0].centroid),
            |x, y| x + y,
        );

        let split_axis = extents.maximum_extent();
        if extents.min[split_axis] == extents.max[split_axis] {
            return None;
        }

        let mid = (primitives.len() - 1) / 2;
        primitives.select_nth_unstable_by(mid, |a, b| {
            a.centroid[split_axis].total_cmp(&b.centroid[split_axis])
        });

        Some(Split {
            axis: split_axis,
            location: mid,
        })
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
