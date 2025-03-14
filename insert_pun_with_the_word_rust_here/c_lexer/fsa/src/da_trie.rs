#[derive(Debug)]
pub struct DATrie {
    base: Vec<i32>,
    next: Vec<i32>,
    check: Vec<i32>,
}

impl DATrie {
    pub fn new() -> Self {
        Self {
            base: vec![],
            next: vec![],
            check: vec![],
        }
    }

    pub fn add(&mut self, node: usize, edges: &[(char, i32)]) {
        if self.base.len() >= node {
            self.base.resize(node + 1, -1);
        }

        let next_len = self.next.len();
        self.base[node] = next_len as i32;

        // rust has so much error handling i feel like letting this bitch
        // panic is wrong but i can't think of what to do better right now
        let max_trans = edges
            .iter()
            .max_by(|(c1, _), (c2, _)| c1.cmp(&c2))
            .unwrap()
            .0 as usize;

        self.next.resize(next_len + max_trans, -1);
        self.check.resize(next_len + max_trans, -1);

        for (c, i) in edges {
            let idx = next_len + (*c as usize);
            self.next[idx] = *i;
            self.check[idx] = node as i32;
        }
    }

    pub fn walk(self, s: usize, c: char) -> Option<i32> {
        let t: usize = self.base[s] as usize + c as usize;

        if self.check[t] == (s as i32) {
            Some(self.next[t])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn new_test_1() {
        let da_trie = DATrie::new();
    }
}
