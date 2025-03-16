use arrayvec::ArrayString;
use std::fmt;
use std::fmt::Write;

pub struct TATrie {
    base: Vec<i32>,
    next: Vec<i32>,
    check: Vec<i32>,
}

impl TATrie {
    pub fn new() -> Self {
        Self {
            base: vec![],
            next: vec![],
            check: vec![],
        }
    }

    pub fn add(&mut self, node: usize, edges: &[(char, i32)]) {
        if self.base.len() <= node {
            self.base.resize(node + 1, -1);
        }

        // !! DEBUG !!
        // println!("node {} len {}", node, self.base.len());
        // !! DEBUG !!

        let next_len = self.next.len();
        self.base[node] = next_len as i32;

        // rust has so much error handling i feel like letting this bitch
        // panic is wrong but i can't think of what to do better right now
        let max_trans = edges
            .iter()
            .max_by(|(c1, _), (c2, _)| c1.cmp(&c2))
            .unwrap()
            .0 as usize;

        // !! DEBUG !!
        println!("{} {}", max_trans, next_len);
        // !! DEBUG !!

        self.next.resize(next_len + max_trans + 1, -1);
        self.check.resize(next_len + max_trans + 1, -1);

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

impl fmt::Debug for TATrie {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::with_capacity(1024);
        let mut txt_buf: ArrayString<127> = ArrayString::new();

        write!(
            &mut output,
            "base len: {}, base {:?}\nnext len: {}, next:",
            self.base.len(),
            self.base,
            self.next.len(),
        )
        .unwrap();

        for (idx, nxt_node) in self.next.iter().enumerate() {
            let owner: i32 = self.check[idx];
            if owner != -1 && *nxt_node != -1 {
                // !! DEBUG !!
                // println!("owner {}", owner);
                // !! DEBUG !!

                let owner_start_idx = self.base[owner as usize];

                write!(
                    &mut txt_buf,
                    ", [{} -({})-> {}]",
                    owner,
                    (idx as i32) - owner_start_idx,
                    *nxt_node
                )
                .unwrap();

                output.push_str(txt_buf.as_str());
                txt_buf.clear();
            }
        }

        f.write_str(&output)
    }
    /* this function writes the desired format using std lib only shit.
    i think it'd be interesting to see if its better than the ArrayString
    from arrayvec in any way, other than it just being included already */

    /*
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();

        // unecessary? yes. but what if i wanna refuse to use the heap
        let mut txt_buf: [u8; 128] = [0; 128];
        let mut txt_buf_cur = Cursor::new(&mut txt_buf[..]);

        // fuck it, let it panic, fuck it
        write!(txt_buf_cur, "next len: {} next: [", self.next.len()).unwrap();
        output.extend(txt_buf_cur.bytes().map(|byte| byte.ok().unwrap() as char));

        for (idx, nxt_node) in self.next.iter().enumerate() {
            if *nxt_node != -1 {
                let owner_start_idx = self.base[idx];
                let owner = self.check[idx];

                let mut txt_buf_cur = Cursor::new(&mut txt_buf[..]);
                write!(
                    txt_buf_cur,
                    "[{} -{}-> {}], ",
                    owner,
                    (idx as i32) - owner_start_idx,
                    nxt_node
                )
                .unwrap();

                output.extend(txt_buf.iter().map(|&byte| byte as char));
            }
        }

        // add base info
        output.push_str("");

        write!(f, "{} is {} years old", 6, 9)
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn da_trie_new_test_1() {
        let mut da_trie = TATrie::new();
        let node0 = [('a', 1), ('b', 0)];
        let node1 = [('a', 2), ('b', 1)];
        let node2 = [('a', 1), ('b', 0)];
        let node3 = [('a', 1), ('b', 0)];

        da_trie.add(0, &node0);
        da_trie.add(1, &node1);
        da_trie.add(2, &node2);
        da_trie.add(3, &node3);

        assert_eq!(da_trie.base.len(), 4);
        println!("{:#?}", da_trie);
    }
}
