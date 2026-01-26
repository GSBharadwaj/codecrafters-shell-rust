pub mod trie {
    use std::collections::HashMap;
    struct TrieNode {
        children: HashMap<char, TrieNode>,
        is_terminal: bool,
    }

    impl TrieNode {
        fn new() -> TrieNode {
            TrieNode {
                children: HashMap::new(),
                is_terminal: false,
            }
        }
    }

    pub struct Trie {
        root: TrieNode
    }

    impl Trie {
        pub fn new<I, S>(words: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: AsRef<str>, // "Anything that can be seen as a &str". This is so damn cool!
        {
            let mut trie = Self {
                root: TrieNode::new(),
            };

            for word in words {
                trie.insert_internal(word.as_ref())
            }

            trie
        }

        fn insert_internal(&mut self, word: &str) {
            let mut cur = &mut self.root;
            for c in word.chars() {
                cur = cur.children.entry(c).or_insert_with(|| TrieNode::new())
            }
            cur.is_terminal = true
        }

        fn prefix_search_dfs_helper(&self, node: &TrieNode, slate: &mut String, results: &mut Vec<String>) {
            if node.is_terminal {
                results.push(slate.clone())
            }

            for (&ch, child) in &node.children {
                slate.push(ch);
                self.prefix_search_dfs_helper(child, slate, results);
                slate.pop();
            }
        }

        pub fn prefix_search(&self, word: &str) -> Vec<String> {
            let mut current = &self.root;
            for ch in word.chars() {
                if !current.children.contains_key(&ch) {
                    return Vec::new()
                }
                current = &current.children[&ch];
            }
            let mut results = Vec::new();
            let mut slate = String::new();
            slate.push_str(word);

            self.prefix_search_dfs_helper(current, &mut slate, &mut results);
            results
        }
    }
}
