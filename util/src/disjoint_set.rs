bitfield::bitfield! {
  #[derive(Clone, Copy)]
  struct DisjointSetElem(u16);
  impl Debug;
  parent_or_size, set_parent_or_size: 14, 0;
  is_root, set_root: 15;
}

impl DisjointSetElem {
    pub fn new() -> DisjointSetElem {
        let mut result = DisjointSetElem(0);
        result.set_parent_or_size(1);
        result.set_root(true);
        result
    }
}

#[derive(Debug, Clone)]
pub struct DisjointSetWithMaxSize {
    parent_or_size: Vec<DisjointSetElem>,
    max_set_size: u16,
}

impl DisjointSetWithMaxSize {
    pub fn new(num_elements: u16) -> DisjointSetWithMaxSize {
        if num_elements >= (u16::MAX << 1) {
            panic!("Maximum number of elements is {}", (u16::MAX << 1) - 1);
        }

        DisjointSetWithMaxSize {
            parent_or_size: vec![DisjointSetElem::new(); num_elements as usize],
            max_set_size: 1,
        }
    }

    pub fn reset(&mut self) {
        self.parent_or_size.fill(DisjointSetElem::new());
        self.max_set_size = 1;
    }

    pub fn find(&mut self, mut elem: u16) -> u16 {
        loop {
            let elem_info = self.parent_or_size[elem as usize];
            if elem_info.is_root() {
                break;
            }

            let parent = elem_info.parent_or_size();
            let parent_info = self.parent_or_size[parent as usize];

            let grandparent = match parent_info.is_root() {
                true => parent,
                false => parent_info.parent_or_size(),
            };

            self.parent_or_size[elem as usize].set_parent_or_size(grandparent);
            elem = grandparent;
        }

        elem
    }

    pub fn union(&mut self, mut lhs: u16, mut rhs: u16) {
        lhs = self.find(lhs);
        rhs = self.find(rhs);

        if lhs == rhs {
            return;
        }

        // Put index with largest set size in lhs.
        debug_assert!(self.parent_or_size[lhs as usize].is_root());
        debug_assert!(self.parent_or_size[rhs as usize].is_root());
        let lhs_size = self.parent_or_size[lhs as usize].parent_or_size();
        let rhs_size = self.parent_or_size[rhs as usize].parent_or_size();
        if lhs_size < rhs_size {
            (lhs, rhs) = (rhs, lhs);
        }

        // Make lhs the new root.
        let rhs_info = &mut self.parent_or_size[rhs as usize];
        rhs_info.set_root(false);
        rhs_info.set_parent_or_size(lhs);

        // Update sizes.
        let union_size = lhs_size + rhs_size;
        self.parent_or_size[lhs as usize].set_parent_or_size(union_size);
        self.max_set_size = self.max_set_size.max(union_size);
    }

    pub fn max_set_size(&self) -> u16 {
        self.max_set_size
    }
}
