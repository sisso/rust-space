use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Tree<K: Hash + Eq + Copy + Clone> {
    parents: HashMap<K, K>,
}

#[derive(Debug, Clone)]
pub struct TreeHier<K: Hash + Eq + Copy + Clone> {
    pub index: K,
    pub parent: Option<K>,
    pub deep: usize,
}

pub struct TreeIterator<'a, K: Hash + Eq + Copy + Clone> {
    _tree: &'a Tree<K>,
    list: VecDeque<TreeHier<K>>,
}

impl<'a, K: Hash + Eq + Copy + Clone> TreeIterator<'a, K> {
    pub fn new(tree: &'a Tree<K>) -> Self {
        fn add_recursive<K: Hash + Eq + Copy + Clone>(
            tree: &Tree<K>,
            list: &mut VecDeque<TreeHier<K>>,
            deep: usize,
            parent: K,
        ) {
            tree.children(parent).for_each(|i| {
                list.push_back(TreeHier {
                    index: i,
                    parent: Some(parent),
                    deep: deep,
                });

                add_recursive(tree, list, deep + 1, i);
            });
        }

        let mut list = VecDeque::new();

        for root in tree.find_roots() {
            list.push_back(TreeHier {
                index: root,
                parent: None,
                deep: 0,
            });
            add_recursive(tree, &mut list, 1, root);
        }

        TreeIterator { _tree: tree, list }
    }
}

impl<'a, K: Hash + Eq + Copy + Clone> Iterator for TreeIterator<'a, K> {
    type Item = TreeHier<K>;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
}

impl<K: Hash + Eq + Copy + Clone> Tree<K> {
    pub fn new() -> Self {
        Tree {
            parents: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, parent: K) -> Option<K> {
        self.parents.insert(key, parent)
    }

    pub fn remove(&mut self, key: K) -> Option<K> {
        self.parents.remove(&key)
    }

    pub fn get(&self, key: K) -> Option<K> {
        self.parents.get(&key).cloned()
    }

    pub fn children<'a>(&'a self, root: K) -> impl Iterator<Item = K> + 'a {
        self.parents
            .iter()
            .filter(move |(_, &value)| value == root)
            .map(|(&key, _)| key)
    }

    pub fn children_deep(&self, root: K) -> Vec<K> {
        let mut buffer = Vec::new();

        for i in self.children(root) {
            buffer.push(i);

            let childrens = self.children_deep(i);
            buffer.extend(childrens);
        }

        buffer
    }

    pub fn parents_inclusive(&self, from: K) -> Vec<K> {
        let mut buffer = vec![from];
        self.find_parents(&mut buffer, from);
        buffer
    }

    /// Not include self
    pub fn parents(&self, from: K) -> Vec<K> {
        let mut buffer = vec![];
        self.find_parents(&mut buffer, from);
        buffer
    }

    pub fn list_all(&self) -> &HashMap<K, K> {
        &self.parents
    }

    pub fn iter_hier<'a>(&'a self) -> impl Iterator<Item = TreeHier<K>> + 'a {
        TreeIterator::new(self)
    }

    fn find_parents(&self, buffer: &mut Vec<K>, from: K) {
        let mut current = from;
        loop {
            let parent = self.get(current);
            match parent {
                Some(location_id) => {
                    buffer.push(location_id);
                    current = location_id;
                }
                None => break,
            }
        }
    }

    fn find_roots(&self) -> Vec<K> {
        let mut roots: HashSet<K> = HashSet::new();

        self.parents
            .values()
            .filter(|i| self.parents(**i).is_empty())
            .copied()
            .for_each(|i| {
                roots.insert(i);
            });

        roots.into_iter().collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sample_tree() -> Tree<i32> {
        let mut tree = Tree::new();
        /*
           0
           +1
           |+2
           ||+5
           | |+6
           |+4
           ||+7
           +3
        */
        tree.insert(1, 0);
        tree.insert(2, 1);
        tree.insert(3, 0);
        tree.insert(4, 1);
        tree.insert(5, 2);
        tree.insert(6, 5);
        tree.insert(7, 4);
        tree
    }

    #[test]
    fn test_tree_children() {
        let tree = sample_tree();

        assert_eq!(tree.get(0), None);
        assert_eq!(tree.get(1), Some(0));
        assert_eq!(tree.get(2), Some(1));
        assert_eq!(tree.get(3), Some(0));

        let mut children: Vec<_> = tree.children(0).collect();
        children.sort();
        assert_eq!(children, vec![1, 3]);
        assert_eq!(tree.children(4).collect::<Vec<_>>(), vec![7]);
        assert!(tree.children(7).next().is_none());
    }

    #[test]
    fn test_tree_children_deep() {
        let tree = sample_tree();

        let tests = vec![
            (0, vec![1, 2, 3, 4, 5, 6, 7]),
            (1, vec![2, 4, 5, 6, 7]),
            (2, vec![5, 6]),
            (3, vec![]),
            (4, vec![7]),
            (5, vec![6]),
            (6, vec![]),
            (7, vec![]),
        ];

        for (index, expected) in tests {
            let mut children = tree.children_deep(index);
            children.sort();
            assert_eq!(children, expected);
        }
    }

    #[test]
    fn test_tree_parents() {
        let tree = sample_tree();

        let tests = vec![
            (0, vec![]),
            (1, vec![0]),
            (2, vec![1, 0]),
            (3, vec![0]),
            (4, vec![1, 0]),
            (5, vec![2, 1, 0]),
            (6, vec![5, 2, 1, 0]),
            (7, vec![4, 1, 0]),
        ];

        for (index, expected) in tests {
            let children = tree.parents(index);
            assert_eq!(children, expected);
        }
    }

    #[test]
    fn test_tree_iter() {
        let mut tree = Tree::new();
        tree.insert(1, 0);
        tree.insert(2, 0);
        tree.insert(3, 1);

        for (id, parent) in tree.list_all() {
            match (id, parent) {
                (1, 0) | (2, 0) | (3, 1) => {}
                other => panic!("unexpected {:?}", other),
            }
        }
    }

    #[test]
    fn test_find_roots() {
        let tree = sample_tree();
        assert_eq!(vec![0], tree.find_roots());
    }

    #[test]
    fn test_tree_iter_empty() {
        let tree = Tree::<u32>::new();
        assert_eq!(0, tree.iter_hier().count());
    }

    #[test]
    fn test_tree_iter_one() {
        let mut tree = Tree::new();
        tree.insert(1, 0);
        assert_tree_iter(&tree, vec![(None, 0, 0), (Some(0), 1, 1)]);
    }

    #[test]
    fn test_tree_iter_two_roots() {
        let mut tree = Tree::new();
        tree.insert(1, 0);
        tree.insert(3, 2);
        assert_tree_iter_set(
            &tree,
            vec![(None, 0, 0), (Some(0), 1, 1), (None, 2, 0), (Some(2), 3, 1)]
                .into_iter()
                .collect(),
        );
    }

    #[test]
    fn test_tree_iter_hier() {
        let mut tree = Tree::new();
        /*
           0
           +1
           |+2
           | +5
           |  +6
           |+4
           | +7
           +3
           8
           +9
           +10
        */
        tree.insert(1, 0);
        tree.insert(2, 1);
        tree.insert(3, 0);
        tree.insert(4, 1);
        tree.insert(5, 2);
        tree.insert(6, 5);
        tree.insert(7, 4);
        tree.insert(9, 8);
        tree.insert(10, 8);

        let expected = vec![
            (None, 0, 0),
            (Some(0), 1, 1),
            (Some(1), 2, 2),
            (Some(2), 5, 3),
            (Some(5), 6, 4),
            (Some(1), 4, 2),
            (Some(4), 7, 3),
            (Some(0), 3, 1),
            (None, 8, 0),
            (Some(8), 9, 1),
            (Some(8), 10, 1),
        ];

        assert_tree_iter_set(&tree, expected.into_iter().collect());
    }

    #[test]
    fn test_tree_iter_sequence_tree() {
        let mut tree = Tree::new();
        /*
           0
           +1
            +2
             +5
        */
        tree.insert(1, 0);
        tree.insert(2, 1);
        tree.insert(5, 2);

        let expected = vec![
            (None, 0, 0),
            (Some(0), 1, 1),
            (Some(1), 2, 2),
            (Some(2), 5, 3),
        ];

        assert_tree_iter(&tree, expected);
    }

    fn assert_tree_iter(tree: &Tree<i32>, expected: Vec<(Option<i32>, i32, usize)>) {
        let list: Vec<TreeHier<i32>> = tree.iter_hier().collect();

        for i in &list {
            println!("{:?}", i);
        }

        for (i, e) in list.iter().enumerate() {
            let (exp_parent, exp_index, exp_deep) = expected[i];
            assert_eq!(exp_index, e.index, "bad index");
            assert_eq!(exp_parent, e.parent, "bad parent");
            assert_eq!(exp_deep, e.deep, "bad deep");
        }

        assert_eq!(expected.len(), list.len());
    }

    fn assert_tree_iter_set(tree: &Tree<i32>, expected: HashSet<(Option<i32>, i32, usize)>) {
        let list: Vec<TreeHier<i32>> = tree.iter_hier().collect();

        for (_, e) in list.iter().enumerate() {
            let value = (e.parent, e.index, e.deep);
            assert!(expected.contains(&value), "could not find {:?}", e);
        }

        assert_eq!(expected.len(), list.len());
    }
}
