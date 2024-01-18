use std::{fmt::Display, sync::Arc};

use crate::{
    bounds::Bounds,
    geometry::{point::Point, Axis},
    intersection::PrimitiveIntersection,
    primitive::Primitive,
    ray::Ray,
};

#[derive(Debug, PartialEq)]
pub enum BvhNode {
    InteriorNode {
        bounds: Bounds,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
        split_axis: Axis,
    },
    LeafNode {
        bounds: Bounds,
        primitives: Vec<Arc<Primitive>>,
    },
}

pub enum SplitMethod {
    Median,
    Middle,
    SAH,
}

#[derive(Debug, PartialEq)]
pub struct Bvh {
    pub root: BvhNode,
}

impl Bvh {
    pub fn new(primitives: Vec<Arc<Primitive>>, split_method: SplitMethod) -> Self {
        let mut primitive_infos: Vec<_> = primitives
            .iter()
            .map(|p| PrimitiveInfo {
                primitive: Arc::clone(p),
                bounds: p.bounds(),
                centroid: p.bounds().centroid(),
            })
            .collect();

        let root = match split_method {
            SplitMethod::Median => BvhNode::from_median_splitting(&mut primitive_infos),
            SplitMethod::Middle => todo!(),
            SplitMethod::SAH => todo!(),
        };

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
                BvhNode::LeafNode { primitives, .. } => {
                    for primitive in primitives {
                        if let Some(intersection) = primitive.intersect(ray) {
                            if current.is_none()
                                || intersection.distance < current.as_ref().unwrap().distance
                            {
                                current = Some(intersection);
                            }
                        }
                    }
                }
                BvhNode::InteriorNode {
                    left,
                    right,
                    split_axis,
                    ..
                } => {
                    if ray.direction[*split_axis] < 0.0 {
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

#[derive(Debug, PartialEq, Clone)]
struct PrimitiveInfo {
    primitive: Arc<Primitive>,
    bounds: Bounds,
    centroid: Point,
}

impl Display for PrimitiveInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.bounds))
    }
}

impl BvhNode {
    fn from_median_splitting(mut primitive_infos: &mut [PrimitiveInfo]) -> BvhNode {
        assert!(primitive_infos.len() > 0);
        let bounds: Bounds = primitive_infos.iter().map(|p| p.bounds).sum();

        match BvhNode::split_at_median(&mut primitive_infos) {
            Some((split_axis, location)) => {
                let (l, r) = primitive_infos.split_at_mut(location);
                BvhNode::InteriorNode {
                    bounds,
                    left: Box::new(BvhNode::from_median_splitting(l)),
                    right: Box::new(BvhNode::from_median_splitting(r)),
                    split_axis,
                }
            }
            None => BvhNode::LeafNode {
                bounds,
                primitives: primitive_infos
                    .iter()
                    .map(|p| Arc::clone(&p.primitive))
                    .collect(),
            },
        }
    }

    fn split_at_median(primitive_infos: &mut [PrimitiveInfo]) -> Option<(Axis, usize)> {
        if primitive_infos.len() <= 4 {
            return None;
        }

        // TODO: This is a very naive implementation that just always picks the
        // median edge. We should use a SAH to find a better split.
        let extents = primitive_infos.iter().map(|p| p.centroid).fold(
            Bounds::new(primitive_infos[0].centroid, primitive_infos[0].centroid),
            |x, y| x + y,
        );

        let split_axis = extents.maximum_extent();
        if extents.min[split_axis] == extents.max[split_axis] {
            return None;
        }

        let mid = (primitive_infos.len() - 1) / 2;
        primitive_infos.select_nth_unstable_by(mid, |a, b| {
            a.centroid[split_axis].total_cmp(&b.centroid[split_axis])
        });

        Some((split_axis, mid))
    }
}

impl Display for BvhNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeafNode { bounds, primitives } => write!(
                f,
                "primitives: ({:?}) bounds: ({:?})",
                primitives
                    .iter()
                    .map(|p| format!("{:?} ", p.bounds()))
                    .collect::<String>(),
                bounds,
            ),
            Self::InteriorNode {
                left,
                right,
                split_axis,
                bounds,
            } => write!(
                f,
                "left: ({}), right: ({}), split: ({:?}), bounds: {:?}",
                left, right, split_axis, bounds
            ),
        }
    }
}
