use std::cmp::Ordering;
use std::error::Error;
use std::fs::File;
use std::io::*;
use std::mem;

#[derive(Debug)]
enum TreeF<A, B> {
    Empty,
    Branch { val: A, left: B, right: B },
}

trait TreeAlgebra<A> where Self: Sized {
    fn combine(TreeF<A, Self>) -> Self;
}

trait TreeCoalgebra<A> where Self: Sized {
    fn separate(Self) -> TreeF<A, Self>;
    fn is_branch(&Self) -> bool;
}

#[derive(Debug)]
struct TreeNode<A>(TreeF<A, Box<TreeNode<A>>>);

impl<A> TreeAlgebra<A> for TreeNode<A> {
    fn combine(input: TreeF<A, TreeNode<A>>) -> TreeNode<A> {
        match input {
            TreeF::Empty => {
                TreeNode(TreeF::Empty)
            },
            TreeF::Branch { val, left, right } => {
                TreeNode(TreeF::Branch {
                    val: val,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            },
        }
    }
}

impl<A> TreeCoalgebra<A> for TreeNode<A> {
    fn separate(input: TreeNode<A>) -> TreeF<A, TreeNode<A>> {
        match input {
            TreeNode(TreeF::Empty) => {
                TreeF::Empty
            },
            TreeNode(TreeF::Branch { val, left, right }) => {
                TreeF::Branch {
                    val: val,
                    left: *left,
                    right: *right,
                }
            },
        }
    }
    fn is_branch(input: &TreeNode<A>) -> bool {
        match *input {
            TreeNode(TreeF::Empty) => false,
            TreeNode(TreeF::Branch {..}) => true,
        }
    }
}

#[derive(Debug)]
struct AnnotatedTreeNode<A, B> {
    annotation: B,
    node: TreeF<A, Box<AnnotatedTreeNode<A, B>>>,
}

impl<A: Clone, B: TreeAlgebra<A> + Copy> TreeAlgebra<A> for AnnotatedTreeNode<A, B> {
    fn combine(input: TreeF<A, AnnotatedTreeNode<A, B>>) -> AnnotatedTreeNode<A, B> {
        match input {
            TreeF::Empty => {
                AnnotatedTreeNode {
                    annotation: TreeAlgebra::combine(TreeF::Empty),
                    node: TreeF::Empty,
                }
            },
            TreeF::Branch { val, left, right } => {
                let new_ann = TreeAlgebra::combine(TreeF::Branch {
                    val: val.clone(),
                    left: left.annotation,
                    right: right.annotation
                });
                AnnotatedTreeNode {
                    annotation: new_ann,
                    node: TreeF::Branch {
                        val: val,
                        left: Box::new(left),
                        right: Box::new(right),
                    }
                }
            },
        }
    }
}

impl<A, B> TreeCoalgebra<A> for AnnotatedTreeNode<A, B> {
    fn separate(input: AnnotatedTreeNode<A, B>) -> TreeF<A, AnnotatedTreeNode<A, B>> {
        match input.node {
            TreeF::Empty => {
                TreeF::Empty
            },
            TreeF::Branch { val, left, right } => {
                TreeF::Branch {
                    val: val,
                    left: *left,
                    right: *right,
                }
            }
        }
    }
    fn is_branch(input: &AnnotatedTreeNode<A, B>) -> bool {
        match input.node {
            TreeF::Empty => false,
            TreeF::Branch {..} => true,
        }
    }
}

// In order to simplify many of the tree operations, we define a zipper type,
// which intuitively represents a location on the tree. To be precise, a zipper
// consists of the following parts:
// 1) A sequence of steps down the tree. Each step contains:
//    a) a direction (left or right)
//    b) the value in the parent node
//    c) the sibling subtree
// 2) The subtree below our location.

#[derive(Debug)]
enum Direction {
    Left,
    Right,
}

#[derive(Debug)]
struct TreeZipperStep<A, B> {
    direction: Direction,
    parent_val: A,
    sibling: B,
}

#[derive(Debug)]
struct TreeZipper<A, B> {
    path: Vec<TreeZipperStep<A, B>>,
    here: B,
}

fn find<A : Ord, B : TreeCoalgebra<A> + TreeAlgebra<A>>(root: B, v: &A) -> TreeZipper<A, B> {
    let mut path = Vec::new();
    let mut node = root;
    loop {
        match TreeCoalgebra::separate(node) {
            TreeF::Empty => {
                node = TreeAlgebra::combine(TreeF::Empty);
                break;
            },
            TreeF::Branch { val, left, right } => {
                match v.cmp(&val) {
                    Ordering::Less => {
                        path.push(TreeZipperStep {
                            direction: Direction::Left,
                            parent_val: val,
                            sibling: right,
                        });
                        node = left;
                    },
                    Ordering::Equal => {
                        node = TreeAlgebra::combine(TreeF::Branch { val: val, left: left, right: right });
                        break;
                    },
                    Ordering::Greater => {
                        path.push(TreeZipperStep {
                            direction: Direction::Right,
                            parent_val: val,
                            sibling: left,
                        });
                        node = right;
                    },
                };
            },
        };
    }
    TreeZipper {
        path: path,
        here: node,
    }
}

fn zip_tree<A, B : TreeAlgebra<A>>(zipper: TreeZipper<A, B>) -> B {
    let mut path = zipper.path;
    let mut node = zipper.here;
    while let Some(TreeZipperStep {direction, parent_val, sibling}) = path.pop() {
        match direction {
            Direction::Left => {
                node = TreeAlgebra::combine(TreeF::Branch {
                    val: parent_val,
                    left: node,
                    right: sibling,
                });
            },
            Direction::Right => {
                node = TreeAlgebra::combine(TreeF::Branch {
                    val: parent_val,
                    left: sibling,
                    right: node,
                });
            },
        }
    }
    node
}

fn root_zipper<A, B>(root: B) -> TreeZipper<A, B> {
    let path = Vec::new();
    TreeZipper {
        path: path,
        here: root,
    }
}

fn parent_zipper<A, B: TreeAlgebra<A>>(zipper: TreeZipper<A, B>) -> TreeZipper<A, B> {
    let mut path = zipper.path;
    match path.pop() {
        None => {
            TreeZipper {
                path: path,
                here: zipper.here,
            }
        },
        Some(TreeZipperStep {direction, parent_val, sibling}) => {
            match direction {
                Direction::Left => {
                    TreeZipper {
                        path: path,
                        here: TreeAlgebra::combine(TreeF::Branch {
                            val: parent_val,
                            left: zipper.here,
                            right: sibling,
                        }),
                    }
                },
                Direction::Right => {
                    TreeZipper {
                        path: path,
                        here: TreeAlgebra::combine(TreeF::Branch {
                            val: parent_val,
                            left: sibling,
                            right: zipper.here,
                        }),
                    }
                },
            }
        },
    }
}

fn left_zipper<A, B: TreeCoalgebra<A> + TreeAlgebra<A>>(zipper: TreeZipper<A, B>) -> TreeZipper<A, B> {
    let mut path = zipper.path;
    let mut node = zipper.here;
    match TreeCoalgebra::separate(node) {
        TreeF::Empty => {
            node = TreeAlgebra::combine(TreeF::Empty);
        },
        TreeF::Branch{val, left, right} => {
            path.push(TreeZipperStep {
                direction: Direction::Left,
                parent_val: val,
                sibling: right,
            });
            node = left;
        },
    }
    TreeZipper {
        path: path,
        here: node,
    }
}

fn right_zipper<A, B: TreeCoalgebra<A> + TreeAlgebra<A>>(zipper: TreeZipper<A, B>) -> TreeZipper<A, B> {
    let mut path = zipper.path;
    let mut node = zipper.here;
    match TreeCoalgebra::separate(node) {
        TreeF::Empty => {
            node = TreeAlgebra::combine(TreeF::Empty);
        },
        TreeF::Branch{val, left, right} => {
            path.push(TreeZipperStep {
                direction: Direction::Right,
                parent_val: val,
                sibling: left,
            });
            node = right;
        },
    }
    TreeZipper {
        path: path,
        here: node,
    }
}

fn rotate_zipper<A, B: TreeCoalgebra<A> + TreeAlgebra<A>>(zipper: TreeZipper<A, B>) -> TreeZipper<A, B> {
    let mut path = zipper.path;
    let mut node = zipper.here;
    match TreeCoalgebra::separate(node) {
        TreeF::Empty => {
            node = TreeAlgebra::combine(TreeF::Empty);
        },
        TreeF::Branch {val, left, right} => {
            if let Some(TreeZipperStep {direction, parent_val, sibling}) = path.pop() {
                match direction {
                    Direction::Left => {
                        node = TreeAlgebra::combine(TreeF::Branch {
                            val: val,
                            left: left,
                            right: TreeAlgebra::combine(TreeF::Branch {
                                val: parent_val,
                                left: right,
                                right: sibling,
                            }),
                        });
                    },
                    Direction::Right => {
                        node = TreeAlgebra::combine(TreeF::Branch {
                            val: val,
                            left: TreeAlgebra::combine(TreeF::Branch {
                                val: parent_val,
                                left: sibling,
                                right: left,
                            }),
                            right: right,
                        });
                    },
                }
            } else {
                node = TreeAlgebra::combine(TreeF::Branch{
                    val: val,
                    left: left,
                    right: right,
                });
            }
        },
    }
    TreeZipper {
        path: path,
        here: node,
    }
}

fn splay<A, B: TreeAlgebra<A> + TreeCoalgebra<A>>(mut zipper: TreeZipper<A, B>) -> TreeZipper<A, B> {
    if zipper.path.is_empty() {
        return zipper
    }
    if !TreeCoalgebra::is_branch(&zipper.here) {
        zipper = parent_zipper(zipper)
    }
    while !zipper.path.is_empty() {
        zipper = splay_step(zipper);
    }
    zipper
}

fn splay_step<A, B: TreeAlgebra<A> + TreeCoalgebra<A>>(zipper: TreeZipper<A, B>) -> TreeZipper<A, B> {
    let mut path = zipper.path;
    match TreeCoalgebra::separate(zipper.here) {
        TreeF::Empty => TreeZipper{path: path, here: TreeAlgebra::combine(TreeF::Empty)},
        TreeF::Branch {val, left, right} => {
            match path.pop() {
                None => {
                    TreeZipper{path: path, here: TreeAlgebra::combine(TreeF::Branch{val: val, left: left, right: right})}
                },
                Some(TreeZipperStep {direction, parent_val, sibling}) => {
                    match path.pop() {
                        None => {
                            match direction {
                                Direction::Left => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeAlgebra::combine(TreeF::Branch {
                                            val: val,
                                            left: left,
                                            right: TreeAlgebra::combine(TreeF::Branch {
                                                val: parent_val,
                                                left: right,
                                                right: sibling,
                                            }),
                                        }),
                                    }
                                },
                                Direction::Right => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeAlgebra::combine(TreeF::Branch {
                                            val: val,
                                            left: TreeAlgebra::combine(TreeF::Branch {
                                                val: parent_val,
                                                left: sibling,
                                                right: left,
                                            }),
                                            right: right,
                                        }),
                                    }
                                },
                            }
                        },
                        Some(TreeZipperStep {direction: parent_dir, parent_val: grandparent_val, sibling: uncle}) => {
                            match (direction, parent_dir) {
                                (Direction::Left, Direction::Left) => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeAlgebra::combine(TreeF::Branch {
                                            val: val,
                                            left: left,
                                            right: TreeAlgebra::combine(TreeF::Branch {
                                                val: parent_val,
                                                left: right,
                                                right: TreeAlgebra::combine(TreeF::Branch {
                                                    val: grandparent_val,
                                                    left: sibling,
                                                    right: uncle,
                                                }),
                                            }),
                                        }),
                                    }
                                },
                                (Direction::Left, Direction::Right) => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeAlgebra::combine(TreeF::Branch {
                                            val: val,
                                            left: TreeAlgebra::combine(TreeF::Branch {
                                                val: grandparent_val,
                                                left: uncle,
                                                right: left,
                                            }),
                                            right: TreeAlgebra::combine(TreeF::Branch {
                                                val: parent_val,
                                                left: right,
                                                right: sibling,
                                            }),
                                        }),
                                    }
                                },
                                (Direction::Right, Direction::Left) => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeAlgebra::combine(TreeF::Branch {
                                            val: val,
                                            left: TreeAlgebra::combine(TreeF::Branch {
                                                val: parent_val,
                                                left: sibling,
                                                right: left,
                                            }),
                                            right: TreeAlgebra::combine(TreeF::Branch {
                                                val: grandparent_val,
                                                left: right,
                                                right: uncle,
                                            }),
                                        }),
                                    }
                                },
                                (Direction::Right, Direction::Right) => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeAlgebra::combine(TreeF::Branch {
                                            val: val,
                                            left: TreeAlgebra::combine(TreeF::Branch {
                                                val: parent_val,
                                                left: TreeAlgebra::combine(TreeF::Branch {
                                                    val: grandparent_val,
                                                    left: uncle,
                                                    right: sibling,
                                                }),
                                                right: left,
                                            }),
                                            right: right,
                                        }),
                                    }
                                },
                            }
                        },
                    }
                },
            }
        },
    }
}

#[derive(Debug)]
struct SplayTree<A> {
    root: TreeNode<A>,
}

trait Splay<A> {
    fn new() -> Self;
    fn insert(&mut self, A);
    fn contains(&mut self, A) -> bool;
    fn splay_to_root(&mut self, A);
}

impl<A: Ord> Splay<A> for SplayTree<A> {
    fn new() -> SplayTree<A> {
        SplayTree {
            root: TreeNode(TreeF::Empty),
        }
    }

    fn insert(&mut self, v: A) {
        let old_root = mem::replace(&mut self.root, TreeNode(TreeF::Empty));
        let mut ins_loc = find(old_root, &v);
        if let TreeNode(TreeF::Empty) = ins_loc.here {
            ins_loc.here = TreeNode(TreeF::Branch {
                val: v,
                left: Box::new(TreeNode(TreeF::Empty)),
                right: Box::new(TreeNode(TreeF::Empty)),
            });
        }
        self.root = zip_tree(splay(ins_loc));
    }

    fn contains(&mut self, v: A) -> bool {
        let old_root = mem::replace(&mut self.root, TreeNode(TreeF::Empty));
        let find_loc = find(old_root, &v);
        let result = match find_loc.here {
            TreeNode(TreeF::Empty) => false,
            TreeNode(TreeF::Branch { .. }) => true,
        };
        self.root = zip_tree(splay(find_loc));
        result
    }

    fn splay_to_root(&mut self, v: A) {
        let old_root = mem::replace(&mut self.root, TreeNode(TreeF::Empty));
        self.root = zip_tree(splay(find(old_root, &v)));
    }
}

#[derive(Debug)]
enum BitRangeNode {
    Empty,
    Branch {
        here: bool,
        size: i32,
        reversed: bool,
        left: Box<BitRangeNode>,
        right: Box<BitRangeNode>,
    }
}

trait Reversible {
    fn reversed(Self) -> Self;
}

impl Reversible for BitRangeNode {
    fn reversed(input: BitRangeNode) -> BitRangeNode {
        match input {
            BitRangeNode::Empty => {
                BitRangeNode::Empty
            },
            BitRangeNode::Branch { here, size, reversed, left, right } => {
                BitRangeNode::Branch {
                    here: here,
                    size: size,
                    reversed: !reversed,
                    left: left,
                    right: right,
                }
            },
        }
    }
}

fn get_size(n: &BitRangeNode) -> i32 {
    match *n {
        BitRangeNode::Empty => 0,
        BitRangeNode::Branch { size: s, .. } => s,
    }
}

impl TreeAlgebra<bool> for BitRangeNode {
    fn combine(input: TreeF<bool, BitRangeNode>) -> BitRangeNode {
        match input {
            TreeF::Empty => {
                BitRangeNode::Empty
            },
            TreeF::Branch { val, left, right } => {
                BitRangeNode::Branch {
                    here: val,
                    size: get_size(&left) + get_size(&right) + 1,
                    reversed: false,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
        }
    }
}

impl TreeCoalgebra<bool> for BitRangeNode {
    fn separate(input: BitRangeNode) -> TreeF<bool, BitRangeNode> {
        match input {
            BitRangeNode::Empty => {
                TreeF::Empty
            },
            BitRangeNode::Branch {here, reversed, left, right, ..} => {
                if reversed {
                    TreeF::Branch {
                        val: here,
                        left: Reversible::reversed(*right),
                        right: Reversible::reversed(*left),
                    }
                } else {
                    TreeF::Branch {
                        val: here,
                        left: *left,
                        right: *right,
                    }
                }
            },
        }
    }
    fn is_branch(input: &BitRangeNode) -> bool {
        match *input {
            BitRangeNode::Empty => false,
            BitRangeNode::Branch {..} => true,
        }
    }
}

fn end<A, B: TreeCoalgebra<A> + TreeAlgebra<A>>(root: B) -> TreeZipper<A, B> {
    let mut node = root;
    let mut path = Vec::new();
    loop {
        match TreeCoalgebra::separate(node) {
            TreeF::Empty => {
                node = TreeAlgebra::combine(TreeF::Empty);
                break;
            },
            TreeF::Branch {val, left, right} => {
                path.push(TreeZipperStep {
                    direction: Direction::Right,
                    parent_val: val,
                    sibling: left,
                });
                node = right;
            }
        }
    }
    TreeZipper {
        here: node,
        path: path,
    }
}

fn find_index(root: BitRangeNode, index: i32) -> TreeZipper<bool, BitRangeNode> {
    let mut node = root;
    let mut remaining = index;
    let mut path = Vec::new();
    loop {
        match TreeCoalgebra::separate(node) {
            TreeF::Empty => {
                node = TreeAlgebra::combine(TreeF::Empty);
                break;
            },
            TreeF::Branch { val, left, right } => {
                let left_size = get_size(&left);
                match left_size.cmp(&remaining) {
                    Ordering::Less => {
                        path.push(TreeZipperStep {
                            direction: Direction::Right,
                            parent_val: val,
                            sibling: left,
                        });
                        node = right;
                        remaining = remaining - left_size - 1;
                    },
                    Ordering::Equal => {
                        node = TreeAlgebra::combine(TreeF::Branch {val: val, left: left, right: right});
                        break;
                    },
                    Ordering::Greater => {
                        path.push(TreeZipperStep {
                            direction: Direction::Left,
                            parent_val: val,
                            sibling: right,
                        });
                        node = left;
                    },
                }
            },
        }
    }
    TreeZipper {
        path: path,
        here: node,
    }
}

fn isolate_interval(root: BitRangeNode, index_start: i32, index_end: i32) -> TreeZipper<bool, BitRangeNode> {
    let mut cur_root = root;
    if index_start <= 0 {
        if index_end >= get_size(&cur_root) {
            root_zipper(cur_root)
        } else {
            left_zipper(splay(find_index(cur_root, index_end)))
        }
    } else {
        if index_end >= get_size(&cur_root) {
            right_zipper(splay(find_index(cur_root, index_start - 1)))
        } else {
            cur_root = zip_tree(splay(find_index(cur_root, index_start)));
            cur_root = zip_tree(splay(find_index(cur_root, index_start-1)));
            cur_root = zip_tree(splay(find_index(cur_root, index_end)));
            let zipper = right_zipper(find_index(cur_root, index_start-1));
            if TreeCoalgebra::is_branch(&zipper.here) {
                zipper
            } else {
                right_zipper(rotate_zipper(parent_zipper(zipper)))
            }
        }
    }
}

#[derive(Debug)]
struct BitRange {
    root: BitRangeNode,
}

impl BitRange {
    fn new(n: i32) -> BitRange {
        let mut root = BitRangeNode::Empty;
        for _ in 0..n {
            let mut zipper = end(root);
            zipper.here = TreeAlgebra::combine(TreeF::Branch {
                val: false,
                left: BitRangeNode::Empty,
                right: BitRangeNode::Empty,
            });
            root = zip_tree(splay(zipper));
        }
        BitRange {
            root: root,
        }
    }

    fn set(self: &mut BitRange, index: i32, val: bool) {
        let old_root = mem::replace(&mut self.root, BitRangeNode::Empty);
        let mut zipper = find_index(old_root, index);
        zipper.here = match zipper.here {
            BitRangeNode::Empty => {
                BitRangeNode::Empty
            },
            BitRangeNode::Branch { size, reversed, left, right, .. } => {
                BitRangeNode::Branch {
                    here: val,
                    size: size,
                    reversed: reversed,
                    left: left,
                    right: right,
                }
            },
        };
        self.root = zip_tree(splay(zipper));
    }

    fn get(self: &mut BitRange, index: i32) -> Option<bool> {
        let old_root = mem::replace(&mut self.root, BitRangeNode::Empty);
        let zipper = find_index(old_root, index);
        let result = match zipper.here {
            BitRangeNode::Empty => None,
            BitRangeNode::Branch { here, .. } => Some(here),
        };
        self.root = zip_tree(splay(zipper));
        result
    }

    fn reverse_range(self: &mut BitRange, index_start: i32, index_end: i32) {
        let tmp_root = mem::replace(&mut self.root, BitRangeNode::Empty);
        let mut zipper = isolate_interval(tmp_root, index_start, index_end);
        zipper.here = Reversible::reversed(zipper.here);
        self.root = zip_tree(zipper);
    }
}

fn main() {
    let fin = match File::open("range_reverse.in") {
        Err(why) => panic!("Could not open input file: {}", why.description()),
        Ok(file) => file,
    };
    let mut fin = BufReader::new(fin);
    let fout = match File::create("range_reverse.out") {
        Err(why) => panic!("Could not open output file: {}", why.description()),
        Ok(file) => file,
    };
    let mut fout = BufWriter::new(fout);

    let mut line1 = String::new();
    match fin.read_line(&mut line1) {
        Err(why) => panic!("Error reading data: {}", why.description()),
        Ok(_) => {},
    };
    let line1_tokens : Vec<&str> = line1.trim_right().split(' ').collect();
    let n = match line1_tokens[0].parse::<i32>() {
        Err(why) => panic!("Error parsing data: {}", why.description()),
        Ok(n) => n,
    };
    let m = match line1_tokens[1].parse::<i32>() {
        Err(why) => panic!("Error parsing data: {}", why.description()),
        Ok(n) => n,
    };
    let mut range = BitRange::new(n);
    for _ in 0..m {
        let mut line = String::new();
        match fin.read_line(&mut line) {
            Err(why) => panic!("Error reading data: {}", why.description()),
            Ok(_) => {},
        };
        let tokens : Vec<&str> = line.trim_right().split(' ').collect();
        if tokens[0] == "S" {
            let idx = match tokens[1].parse::<i32>() {
                Err(why) => panic!("Error parsing data: {}", why.description()),
                Ok(n) => n,
            };
            let val = tokens[2] == "1";
            range.set(idx, val);
        } else if tokens[0] == "G" {
            let idx = match tokens[1].parse::<i32>() {
                Err(why) => panic!("Error parsing data: {}", why.description()),
                Ok(n) => n,
            };
            let val = match range.get(idx) {
                None => panic!("Requested index out of range!"),
                Some(b) => b,
            };
            write!(fout, "{}\n", if val { 1 } else { 0 }).unwrap();
        } else if tokens[0] == "R" {
            let idx1 = match tokens[1].parse::<i32>() {
                Err(why) => panic!("Error parsing data: {}", why.description()),
                Ok(n) => n,
            };
            let idx2 = match tokens[2].parse::<i32>() {
                Err(why) => panic!("Error parsing data: {}", why.description()),
                Ok(n) => n,
            };
            range.reverse_range(idx1, idx2);
        }
    }
}
