use std::cmp::Ordering;
use std::mem;

#[derive(Debug)]
enum TreeF<A, B> {
    Empty,
    Branch { val: A, left: B, right: B },
}

#[derive(Debug)]
struct TreeNode<A>(TreeF<A, Box<TreeNode<A>>>);

// In order to simplify many of the tree operations, we define a zipper type,
// which intuitively represents a location on the tree. To be precise, a zipper
// consists of the following parts:
// 1) A sequence of steps down the tree. Each step contains:
//    a) a direction (left or right)
//    b) the value in the parent node
//    c) the sibling subtree
// 2) The subtree below our location.

enum Direction {
    Left,
    Right,
}

struct TreeZipperStep<A> {
    direction: Direction,
    parent_val: A,
    sibling: TreeNode<A>,
}

struct TreeZipper<A> {
    path: Vec<TreeZipperStep<A>>,
    here: TreeNode<A>,
}

fn find<A : Ord>(root: TreeNode<A>, v: &A) -> TreeZipper<A> {
    let mut path = Vec::new();
    let mut node = root;
    loop {
        match node {
            TreeNode(TreeF::Empty) => break,
            TreeNode(TreeF::Branch { val, left, right }) => {
                match v.cmp(&val) {
                    Ordering::Less => {
                        path.push(TreeZipperStep {
                            direction: Direction::Left,
                            parent_val: val,
                            sibling: *right,
                        });
                        node = *left;
                    },
                    Ordering::Equal => {
                        node = TreeNode(TreeF::Branch { val: val, left: left, right: right });
                        break;
                    },
                    Ordering::Greater => {
                        path.push(TreeZipperStep {
                            direction: Direction::Right,
                            parent_val: val,
                            sibling: *left,
                        });
                        node = *right;
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

fn zip_tree<A>(zipper: TreeZipper<A>) -> TreeNode<A> {
    let mut path = zipper.path;
    let mut node = zipper.here;
    while let Some(TreeZipperStep {direction, parent_val, sibling}) = path.pop() {
        match direction {
            Direction::Left => {
                node = TreeNode(TreeF::Branch {
                    val: parent_val,
                    left: Box::new(node),
                    right: Box::new(sibling),
                });
            },
            Direction::Right => {
                node = TreeNode(TreeF::Branch {
                    val: parent_val,
                    left: Box::new(node),
                    right: Box::new(sibling),
                });
            },
        }
    }
    node
}

fn parent_zipper<A>(zipper: TreeZipper<A>) -> TreeZipper<A> {
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
                        here: TreeNode(TreeF::Branch {
                            val: parent_val,
                            left: Box::new(zipper.here),
                            right: Box::new(sibling),
                        }),
                    }
                },
                Direction::Right => {
                    TreeZipper {
                        path: path,
                        here: TreeNode(TreeF::Branch {
                            val: parent_val,
                            left: Box::new(sibling),
                            right: Box::new(zipper.here),
                        }),
                    }
                },
            }
        },
    }
}

fn splay<A>(mut zipper: TreeZipper<A>) -> TreeZipper<A> {
    if zipper.path.is_empty() {
        return zipper
    }
    match zipper.here {
        TreeNode(TreeF::Empty) => splay(parent_zipper(zipper)),
        TreeNode(TreeF::Branch { .. }) => {
            while !zipper.path.is_empty() {
                zipper = splay_step(zipper);
            }
            zipper
        },
    }
}

fn splay_step<A>(zipper: TreeZipper<A>) -> TreeZipper<A> {
    let mut path = zipper.path;
    match zipper.here {
        TreeNode(TreeF::Empty) => TreeZipper{path: path, here: zipper.here},
        TreeNode(TreeF::Branch {val, left, right}) => {
            match path.pop() {
                None => {
                    TreeZipper{path: path, here: TreeNode(TreeF::Branch{val: val, left: left, right: right})}
                },
                Some(TreeZipperStep {direction, parent_val, sibling}) => {
                    match path.pop() {
                        None => {
                            match direction {
                                Direction::Left => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeNode(TreeF::Branch {
                                            val: val,
                                            left: left,
                                            right: Box::new(TreeNode(TreeF::Branch {
                                                val: parent_val,
                                                left: right,
                                                right: Box::new(sibling),
                                            })),
                                        }),
                                    }
                                },
                                Direction::Right => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeNode(TreeF::Branch {
                                            val: val,
                                            left: Box::new(TreeNode(TreeF::Branch {
                                                val: parent_val,
                                                left: Box::new(sibling),
                                                right: left,
                                            })),
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
                                        here: TreeNode(TreeF::Branch {
                                            val: val,
                                            left: left,
                                            right: Box::new(TreeNode(TreeF::Branch {
                                                val: parent_val,
                                                left: right,
                                                right: Box::new(TreeNode(TreeF::Branch {
                                                    val: grandparent_val,
                                                    left: Box::new(sibling),
                                                    right: Box::new(uncle),
                                                })),
                                            })),
                                        }),
                                    }
                                },
                                (Direction::Left, Direction::Right) => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeNode(TreeF::Branch {
                                            val: val,
                                            left: Box::new(TreeNode(TreeF::Branch {
                                                val: grandparent_val,
                                                left: Box::new(uncle),
                                                right: left,
                                            })),
                                            right: Box::new(TreeNode(TreeF::Branch {
                                                val: parent_val,
                                                left: right,
                                                right: Box::new(sibling),
                                            })),
                                        }),
                                    }
                                },
                                (Direction::Right, Direction::Left) => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeNode(TreeF::Branch {
                                            val: val,
                                            left: Box::new(TreeNode(TreeF::Branch {
                                                val: parent_val,
                                                left: Box::new(sibling),
                                                right: left,
                                            })),
                                            right: Box::new(TreeNode(TreeF::Branch {
                                                val: grandparent_val,
                                                left: right,
                                                right: Box::new(uncle),
                                            })),
                                        }),
                                    }
                                },
                                (Direction::Right, Direction::Right) => {
                                    TreeZipper {
                                        path: path,
                                        here: TreeNode(TreeF::Branch {
                                            val: val,
                                            left: Box::new(TreeNode(TreeF::Branch {
                                                val: parent_val,
                                                left: Box::new(TreeNode(TreeF::Branch {
                                                    val: grandparent_val,
                                                    left: Box::new(uncle),
                                                    right: Box::new(sibling),
                                                })),
                                                right: left,
                                            })),
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

fn main() {
    let mut tree : SplayTree<u8> = Splay::new();
    tree.insert(1);
    tree.insert(2);
    tree.insert(3);
    tree.insert(4);
    tree.insert(5);
    tree.insert(6);
    tree.insert(7);
    tree.insert(8);
    println!("{:?}", tree);
    println!("{:?}", tree.contains(0));
    println!("{:?}", tree);
    println!("{:?}", tree.contains(4));
    println!("{:?}", tree);
}
