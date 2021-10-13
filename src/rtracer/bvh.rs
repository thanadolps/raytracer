use crate::rtracer::bvh::BVHChild::Leaf;
use crate::rtracer::scene::SceneObjectIndex;
use crate::rtracer::shape::Shape;
use crate::rtracer::SceneObject;
use crate::utils::aabb::AABB;
use crate::utils::cell_vec::CellVec;

use typed_index_collections::TiVec;

fn calculate_capacity(mut n: usize) -> usize {
    let mut c = 0;
    while n > 0 {
        c += n;
        n /= 2;
    }
    c
}

#[derive(Clone, Debug)]
pub enum BVHChild {
    Node { left: usize, right: usize },
    Leaf(SceneObjectIndex),
}

#[derive(Clone, Debug)]
pub struct BVHNode {
    pub bounding_box: AABB,
    pub child: BVHChild,
}

/// This structure most likely will behave incorrectly if the slice used to generate this tree
/// get modify/reorder
pub struct BVHTree {
    data: Vec<BVHNode>,
    //    buffer: CellVec<usize>,
}

impl BVHTree {
    fn new(flat_tree: Vec<BVHNode>) -> Self {
        let capacity = calc_dfs_required_capacity(&flat_tree);
        BVHTree {
            data: flat_tree,
            //           buffer: CellVec::new(capacity),
        }
    }

    pub fn flat_tree(&self) -> &Vec<BVHNode> {
        &self.data
    }

    // partition pattern: left[..len/2], right[len/2..]
    pub fn generate(objects: impl Iterator<Item = (SceneObjectIndex, AABB)>) -> Self {
        // TODO: preallocated/calculated capacity
        let mut flat_tree = Vec::new();

        // initial filling
        flat_tree.extend(objects.map(|(index, bb)| BVHNode {
            bounding_box: bb,
            child: Leaf(index),
        }));

        recursive_build_tree_entry(&mut flat_tree);
        BVHTree::new(flat_tree)

        /*
        // TODO: unchecked indices?
        let capacity = calculate_capacity(objects.len());
        let mut flat_tree = Vec::with_capacity(capacity);

        for (i, obj) in objects.iter().enumerate() {
            let bb = obj.shape.bounding_box();
            flat_tree.push(BVHNode {
                domain: bb,
                child: BVHChild::Leaf(i)
            });
        }


        // TODO: removed with certain
        assert_eq!(flat_tree.len(), capacity);*/
    }

    pub fn from_scene_objects<U, I>(
        scene_objs: I,
    ) -> (Self, TiVec<SceneObjectIndex, SceneObject>, U)
    where
        U: Default + Extend<SceneObject>,
        I: IntoIterator<Item = SceneObject>,
    {
        let mut flat_tree = Vec::new();
        let mut bounded_obj = TiVec::new();
        let mut unbounded_obj = U::default();

        for obj in scene_objs {
            if let Some(bb) = obj.shape.bounding_box() {
                let idx = bounded_obj.push_and_get_key(obj);
                flat_tree.push(BVHNode {
                    bounding_box: bb,
                    child: Leaf(idx),
                })
            } else {
                unbounded_obj.extend(Some(obj));
            }
        }

        recursive_build_tree_entry(&mut flat_tree);
        (BVHTree::new(flat_tree), bounded_obj, unbounded_obj)
    }

    pub fn query_leaf<'a>(
        &'a self,
        pruner: impl Fn(&'a BVHNode) -> bool + 'a,
        buffer: &'a mut Vec<usize>,
    ) -> impl Iterator<Item = SceneObjectIndex> + 'a {
        // TODO: unchecked indexing & pushing
        let flat_tree = &self.data;
        buffer.clear();

        // buffer.clear_fast();

        // saftly: should have enough capacity

        // initial filling (with pruning)
        if flat_tree.last().map_or(false, |n| !pruner(n)) {
            buffer.push(flat_tree.len() - 1);
        }

        std::iter::from_fn(move || {
            // depth first search
            loop {
                // TODO: use typed index to reduce unsafe usage
                // TODO: testing pruning
                let i = buffer.pop()?;
                let node = unsafe { flat_tree.get_unchecked(i) };
                match &node.child {
                    &BVHChild::Node { left, right } => {
                        // node expansion + pruning
                        if !pruner(unsafe { flat_tree.get_unchecked(right) }) {
                            buffer.push(right);
                        }
                        if !pruner(unsafe { flat_tree.get_unchecked(left) }) {
                            buffer.push(left);
                        }
                    }
                    // yielding
                    Leaf(id) => break Some(id.clone()),
                }
            }
        })
    }
}

// TODO: better sorting function (generic?)
fn recursive_build_tree_entry(arr: &mut Vec<BVHNode>) {
    let len = arr.len();
    if len > 1 {
        recursive_build_tree(arr, 0, len);
    }
}

fn recursive_build_tree(arr: &mut Vec<BVHNode>, start: usize, end: usize) -> usize {
    let slice = &mut arr[start..end];
    if slice.len() == 1 {
        return start;
    }
    // TODO: Generalize sorting
    slice.sort_unstable_by(|node_a, node_b| {
        node_a
            .bounding_box
            .min()
            .partial_cmp(&node_b.bounding_box.max())
            .unwrap_or(std::cmp::Ordering::Less)
    });

    let mid = (start + end) / 2;
    let left = recursive_build_tree(arr, start, mid);
    let right = recursive_build_tree(arr, mid, end);

    let bounding_box = arr[left].bounding_box.union(&arr[right].bounding_box);
    let child = { BVHChild::Node { left, right } };

    let new_node = BVHNode {
        bounding_box,
        child,
    };

    let i = arr.len();
    arr.push(new_node);
    i
}

// calculate a upper bounded of capacity of buffer required to perform dfs on flat tree
fn calc_dfs_required_capacity(flat_tree: &[BVHNode]) -> usize {
    if flat_tree.is_empty() {
        return 0;
    }

    let mut buffer = vec![(flat_tree.len() - 1, 1)];
    let mut max_depth = 1;

    // dfs, with tracked depth
    while let Some((i, depth)) = buffer.pop() {
        match flat_tree[i].child {
            BVHChild::Node { left, right } => {
                let child_depth = depth + 1;
                buffer.push((left, child_depth));
                buffer.push((right, child_depth));
            }
            Leaf(_) => {
                max_depth = usize::max(max_depth, depth);
            }
        }
    }

    max_depth
}
/*
#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;
    use std::collections::HashSet;

    #[test]
    fn build_tree() {
        let mut d = vec![
            BVHNode {
                bounding_box: AABB::new(Point3::origin(), Point3::new(1.0, 1.0, 1.0)),
                child: BVHChild::Leaf(0),
            },
            BVHNode {
                bounding_box: AABB::new(Point3::origin(), Point3::new(2.0, 1.0, -1.0)),
                child: BVHChild::Leaf(1),
            },
        ];
        recursive_build_tree_entry(&mut d);
    }
}*/
