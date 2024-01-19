use std::{fmt::Display, ops::Add, sync::Arc};

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
            SplitMethod::SAH => BvhNode::from_sah_splitting(primitive_infos),
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

#[derive(Debug, Clone, Copy)]
struct SAHBucket {
    bounds: Bounds,
    count: usize,
}

impl Add for &SAHBucket {
    type Output = SAHBucket;

    fn add(self, rhs: Self) -> Self::Output {
        SAHBucket {
            bounds: self.bounds + rhs.bounds,
            count: self.count + rhs.count,
        }
    }
}

impl BvhNode {
    fn leaf_node(primitive_infos: &[PrimitiveInfo]) -> BvhNode {
        let bounds: Bounds = primitive_infos.iter().map(|p| p.bounds).sum();
        BvhNode::LeafNode {
            bounds,
            primitives: primitive_infos
                .iter()
                .map(|pi| Arc::clone(&pi.primitive))
                .collect(),
        }
    }

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
            None => BvhNode::leaf_node(primitive_infos),
        }
    }

    fn split_at_median(primitive_infos: &mut [PrimitiveInfo]) -> Option<(Axis, usize)> {
        if primitive_infos.len() <= 4 {
            return None;
        }

        let centroid_bounds: Bounds = primitive_infos
            .iter()
            .map(|p| Bounds::new(p.centroid, p.centroid))
            .sum();

        let split_axis = centroid_bounds.maximum_extent();
        if centroid_bounds.min[split_axis] == centroid_bounds.max[split_axis] {
            return None;
        }

        let mid = (primitive_infos.len() - 1) / 2;
        primitive_infos.select_nth_unstable_by(mid, |a, b| {
            a.centroid[split_axis].total_cmp(&b.centroid[split_axis])
        });

        Some((split_axis, mid))
    }

    // Surface Area Heuristic

    fn from_sah_splitting(primitive_infos: Vec<PrimitiveInfo>) -> BvhNode {
        const NUM_BUCKETS: usize = 12;
        const TRAVERSAL_TO_INTERSECTION_COST_RATIO: f64 = 1.0 / 8.0;
        const MAX_LEAF_PRIMITIVES: usize = 4;

        let bounds: Bounds = primitive_infos.iter().map(|i| i.bounds).sum();
        if primitive_infos.len() <= 1 {
            return BvhNode::leaf_node(&primitive_infos);
        }

        let total_surface_area = bounds.surface_area();
        if total_surface_area == 0.0 {
            println!(
                "Encountered primitives with no surface area: {:#?}",
                primitive_infos
            );
            return BvhNode::leaf_node(&primitive_infos);
        }

        let centroid_bounds: Bounds = primitive_infos
            .iter()
            .map(|p| Bounds::new(p.centroid, p.centroid))
            .sum();
        let split_axis = centroid_bounds.maximum_extent();

        // We will assign primitives to buckets that their centroids lie in.
        // Some buckets will be left empty, which is represented with None.
        let mut buckets: Vec<Option<SAHBucket>> = (0..NUM_BUCKETS).map(|_| None).collect();
        let get_bucket_idx = |p: &PrimitiveInfo| {
            let centroid_offset = centroid_bounds.offset(&p.centroid)[split_axis];
            let idx = (NUM_BUCKETS as f64 * centroid_offset) as usize;
            // Include the centroid furthest to the right into the last bucket
            idx.min(NUM_BUCKETS - 1)
        };

        // Add primitives to buckets
        for primitive_info in primitive_infos.iter() {
            let bucket_idx: usize = get_bucket_idx(&primitive_info);
            buckets[bucket_idx] = buckets[bucket_idx]
                .as_ref()
                .and_then(|bucket| {
                    Some(SAHBucket {
                        bounds: bucket.bounds + primitive_info.bounds,
                        count: bucket.count + 1,
                    })
                })
                .or(Some(SAHBucket {
                    bounds: primitive_info.bounds,
                    count: 1,
                }));
        }

        // Calculate costs of splitting at each bucket (except the last, since
        // that would assign everything to the left node)
        let mut costs = [0.0; NUM_BUCKETS - 1];
        for i in 0..NUM_BUCKETS - 1 {
            let mut cost = TRAVERSAL_TO_INTERSECTION_COST_RATIO;
            let (left, right) = buckets.split_at(i + 1);
            for part in [left, right] {
                let mut merged_bucket: Option<SAHBucket> = None;
                for bucket in part {
                    if let Some(bucket) = bucket {
                        merged_bucket = merged_bucket
                            .and_then(|merged_bucket| Some(&merged_bucket + &bucket))
                            .or(Some(*bucket));
                    }
                }
                if let Some(SAHBucket { bounds, count }) = merged_bucket {
                    cost += count as f64 * bounds.surface_area() / total_surface_area;
                }
            }
            // Guard against bad calculations / degenerate cases
            assert!(cost.is_finite());
            costs[i] = cost;
        }

        let mut min_cost_bucket_idx = 0;
        for (i, c) in costs.iter().enumerate() {
            if *c < costs[min_cost_bucket_idx] {
                min_cost_bucket_idx = i;
            }
        }

        // Create a leaf if it will cost less and not have too many primitives
        let leaf_cost = primitive_infos.len() as f64;
        if leaf_cost <= costs[min_cost_bucket_idx] && primitive_infos.len() <= MAX_LEAF_PRIMITIVES {
            return BvhNode::LeafNode {
                bounds,
                primitives: primitive_infos
                    .iter()
                    .map(|p| Arc::clone(&p.primitive))
                    .collect(),
            };
        }

        // Otherwise, split at the minimum cost bucket

        // TODO: construction would be faster if we could in-place partition the
        // slice (a la std::partition in C++), but rust std does not seem to
        // have that.
        let (left, right): (Vec<_>, Vec<_>) =
            primitive_infos.into_iter().partition(|primitive_info| {
                let bucket_idx = get_bucket_idx(&primitive_info);
                bucket_idx <= min_cost_bucket_idx
            });

        assert!(left.len() > 0);
        assert!(right.len() > 0);

        BvhNode::InteriorNode {
            bounds,
            left: Box::new(Self::from_sah_splitting(left)),
            right: Box::new(Self::from_sah_splitting(right)),
            split_axis,
        }
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
