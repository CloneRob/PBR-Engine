// Standard Library
use std::fmt::Debug;
use std::mem;

// External Library
use cgmath::*;
use collision::*;

use accelerator::{Surround, MinVolume, OctreeItem};

impl Surround for Aabb3<f32> {
    fn surrounds(&self, other: &Self) -> bool {
        self.min[0] <= other.min[0] && self.max[0] >= other.max[0] &&
        self.min[1] <= other.min[1] && self.max[1] >= other.max[1] &&
        self.min[2] <= other.min[2] && self.max[2] >= other.max[2]
    }
}


impl MinVolume for Aabb3<f32> {
    // The minimum sidelenght has a large impact on performance
    // It should be evaluated for the project
    fn is_min(&self) -> bool {
        if (self.max.x - self.min.x) <= 1.0 {
            true
        } else {
            false
        }
    }
}

struct DfsIter<'a, I>
    where I: OctreeItem + Clone + Debug + 'a
{
    stack: Vec<&'a OctreeNode<I>>,
}


impl<'a, I> DfsIter<'a, I> where I: OctreeItem + Clone + Debug + 'a
{
    pub fn new(node: &'a OctreeNode<I>) -> DfsIter<'a, I> {
        DfsIter { stack: vec![node] }
    }
}

impl<'a, I> Iterator for DfsIter<'a, I> where I: OctreeItem + Clone + Debug + 'a
{
    type Item = &'a OctreeNode<I>;
    fn next(&mut self) -> Option<&'a OctreeNode<I>> {
        if self.stack.is_empty() {
            None
        } else {
            let node = self.stack.pop().expect("How is this possible!");
            if let NodeType::Internal(ref vec) = node.node_type {
                for child in vec.iter() {
                    self.stack.push(child);
                }
            }
            Some(node)
        }
    }
}

#[derive(Clone)]
enum NodeType<I>
    where I: OctreeItem + Clone
{
    Internal(Vec<OctreeNode<I>>),
    Leaf(I),
    MinLeaf(Vec<I>),
    Empty,
}

#[derive(Clone)]
pub struct OctreeNode<I>
    where I: OctreeItem + Clone
{
    aabb: Aabb3<f32>,
    node_type: NodeType<I>,
}

impl<'a, I> OctreeNode<I> where I: OctreeItem + Clone + Debug
{
    fn internal(aabb: Aabb3<f32>) -> OctreeNode<I> {
        OctreeNode {
            aabb: aabb,
            node_type: NodeType::Internal(Vec::new()),
        }
    }

    pub fn leaf(aabb: Aabb3<f32>) -> OctreeNode<I> {
        OctreeNode {
            aabb: aabb,
            node_type: NodeType::Empty,
        }
    }

    #[inline]
    fn subdivide(&mut self) -> Option<I> {
        let item = if let NodeType::Leaf(ref item) = self.node_type {
            item.clone()
        } else {
            return None;
        };

        let min = self.aabb.min;
        let max = self.aabb.max;
        let side_length = (max.x - min.x) / 2.0;

        // Change current NodeType to internal and create the empty child leaves
        self.node_type = NodeType::Internal(vec![
         OctreeNode::leaf(Aabb3::new(Point3::new(min.x,               min.y,               min.z), 		Point3::new(min.x + side_length, min.y + side_length, min.z + side_length))),
         OctreeNode::leaf(Aabb3::new(Point3::new(min.x + side_length, min.y,               min.z), 		Point3::new(max.x,      		 min.y + side_length, min.z + side_length))),
         OctreeNode::leaf(Aabb3::new(Point3::new(min.x,               min.y + side_length, min.z), 		Point3::new(min.x + side_length, max.y,      		  min.z + side_length))),
         OctreeNode::leaf(Aabb3::new(Point3::new(min.x + side_length, min.y + side_length, min.z), 		Point3::new(max.x,      		 max.y,      		  min.z + side_length))),

         OctreeNode::leaf(Aabb3::new(Point3::new(min.x,               min.y,               min.z + side_length), 	Point3::new(min.x + side_length, min.y + side_length, max.z))),
         OctreeNode::leaf(Aabb3::new(Point3::new(min.x + side_length, min.y,               min.z + side_length), 	Point3::new(max.x,      		 min.y + side_length, max.z))),
         OctreeNode::leaf(Aabb3::new(Point3::new(min.x,               min.y + side_length, min.z + side_length), 	Point3::new(min.x + side_length, max.y,      		  max.z))),
         OctreeNode::leaf(Aabb3::new(Point3::new(min.x + side_length, min.y + side_length, min.z + side_length), 	Point3::new(max.x,      		 max.y,      		  max.z))),
         ]);
        Some(item)
    }

    // Maybe change return value to Result<Ok(_), InsertError<item>>
    #[inline]
    pub fn insert(&mut self, item: I) -> bool {
        if !self.aabb.contains(item.index()) {
            println!("Item not in volume");
            return false;
        }
        let mut stack = vec![self];
        while !stack.is_empty() {
            let node = stack.pop().unwrap();
            if let NodeType::Internal(ref mut vec) = node.node_type {
                let it = vec.iter_mut().find(|x| x.aabb.contains(item.index()) == true);
                if let Some(n) = it {
                    stack.push(n);
                }
            } else {
                let node_type = mem::replace(&mut node.node_type, NodeType::Empty);
                match node_type {
                    NodeType::Leaf(leaf_item) => {
                        if !leaf_item.is_equal(&item) {
                            if !node.aabb.is_min() {
                                node.subdivide();
                                node.insert(leaf_item);
                                stack.push(node);
                            } else {
                                node.node_type = NodeType::MinLeaf(vec![leaf_item, item]);
                                return true;
                            }
                        }
                    }
                    NodeType::MinLeaf(mut vec) => {
                        vec.push(item);
                        node.node_type = NodeType::MinLeaf(vec);
                        return true;
                    }
                    NodeType::Empty => {
                        node.node_type = NodeType::Leaf(item);
                        return true;
                    }
                    _ => panic!("Reached unreachable Match arm in octreenode... how?"),
                }
            }
        }
        false
    }

    #[inline]
    fn in_volume(&'a self, vol: Aabb3<f32>) -> Vec<&'a I> {
        let iter = DfsIter::new(self);
        let mut items = Vec::new();

        for node in iter {
            // only check nodes whose axis aligned bounding box  is inside
            // the given box
            if vol.surrounds(&node.aabb) {
                match node.node_type {
                    NodeType::Leaf(ref item) => {
                        items.push(item);
                    }
                    NodeType::MinLeaf(ref vec) => {
                        for i in vec {
                            items.push(i);
                        }
                    }
                    _ => {}
                }
            }
        }
        items
    }

    #[inline]
    pub fn frustum_culling(&self, frustum: &Frustum<f32>) -> Vec<&I> {
        let mut inside_frustum: Vec<&I> = Vec::new();
        let iter = DfsIter::new(self);
        for node in iter {
            match node.node_type {
                NodeType::Leaf(ref item) => {
                    if item.in_frustum(&frustum) {
                        inside_frustum.push(item);
                    }
                }
                NodeType::MinLeaf(ref vec) => {
                    for item in vec.iter() {
                        if item.in_frustum(&frustum) {
                            inside_frustum.push(item);
                        }
                    }
                }
                _ => {}
            }
        }
        inside_frustum
    }

    #[inline]
    pub fn frustum_culling_cap(&self, frustum: &Frustum<f32>, expected_capacity: usize) -> Vec<&I> {
        let mut inside_frustum: Vec<&I> = Vec::with_capacity(expected_capacity);
        let iter = DfsIter::new(self);
        for node in iter {
            match node.node_type {
                NodeType::Leaf(ref item) => {
                    if item.in_frustum(&frustum) {
                        inside_frustum.push(item);
                    }
                }
                NodeType::MinLeaf(ref vec) => {
                    for item in vec.iter() {
                        if item.in_frustum(&frustum) {
                            inside_frustum.push(item);
                        }
                    }
                }
                _ => {}
            }
        }
        inside_frustum
    }

    fn len(&self) -> usize {
        let mut len: usize = 0;
        match self.node_type {
            NodeType::Internal(ref vec) => {
                len += vec.len();
                for ref node in vec.iter() {
                    len += node.len();
                }
            }
            _ => {}
        }
        len
    }

    pub fn members(&self) -> usize {
        let mut member = 0;
        match self.node_type {
            NodeType::Internal(ref vec) => {
                for ref node in vec.iter() {
                    member += node.members();
                }
            }
            NodeType::Leaf(_) => {
                member += 1;
            }
            NodeType::MinLeaf(ref vec) => {
                member += vec.len();
            }
            NodeType::Empty => {}
        }
        member
    }

    pub fn print_leaf(&self, depth: u16) {
        match self.node_type {
            NodeType::Internal(ref vec) => {
                for node in vec.iter() {
                    node.print_leaf(depth + 1);
                }
            }
            NodeType::Leaf(ref item) => {
                println!("{:?}, at level {:?}", item, depth);
            }
            NodeType::MinLeaf(ref vec) => {
                println!("\nMinLeaf at level {:?}", depth);
                for item in vec {
                    println!("{:?}", item);
                }
                println!("");
            }
            _ => {}
        }
    }

    fn items_in_leaf(&self) -> usize {
        let mut items = 0;
        let iter = DfsIter::new(self);
        for item in iter {
            match item.node_type {
                NodeType::Leaf(_) => {
                    items += 1;
                }
                _ => {}
            }
        }
        items
    }

    fn items_in_min_leaf(&self) -> usize {
        let mut items = 0;
        let iter = DfsIter::new(self);
        for item in iter {
            match item.node_type {
                NodeType::MinLeaf(ref vec) => {
                    items += vec.len();
                }
                _ => {}
            }
        }
        items
    }

    pub fn print_volume_by_level(&self, level: u32) {
        match self.node_type {
            NodeType::Internal(ref vec) => {
                let node_it = vec.iter();
                println!("------------------------------------------");
                println!("Level {:?}\n", level);

                for n in node_it {
                    print!("{:?}", n.aabb);
                    match n.node_type {
                        NodeType::Internal(_) => {
                            println!(" Internal");
                        }
                        NodeType::Leaf(ref item) => {
                            println!(" Leaf containing {:?}", item);
                        }
                        NodeType::MinLeaf(ref vec) => {
                            println!("MinLeaf containing {:?}", vec);
                        }
                        NodeType::Empty => {
                            println!(" Empty Leaf");
                        }
                    }
                }
                let node_it = vec.iter();
                for n in node_it {
                    n.print_volume_by_level(level + 1);
                }
            }
            _ => {}
        }
    }
}
