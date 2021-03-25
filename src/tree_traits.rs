use super::*;

impl std::cmp::PartialOrd for RellTree
{
    /* We will say A(self) ≤ B(other), if A contains at least as much information as B
     * i.e if B is a subgraph of A which has the same root.
     */
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
    {
        let mut node_pairs = vec![(RellTree::NID_ROOT, RellTree::NID_ROOT)];
        while !node_pairs.is_empty()
        {
            let (a_nid, b_nid) = node_pairs.pop().unwrap();
            let a_node = self.nodes.get(&a_nid).unwrap();
            let b_node = other.nodes.get(&b_nid).unwrap();

            match (&b_node.edge, &a_node.edge)
            {
                (RellE::NonExclusive(b_emap), RellE::NonExclusive(a_emap)) =>
                {
                    // If both are non-exclusive, all edges in B must also exist
                    // in A
                    for b_sid in b_emap.keys()
                    {
                        if !a_emap.contains_key(b_sid)
                        {
                            return Some(std::cmp::Ordering::Greater);
                        }

                        let b_nid = b_emap.get(b_sid).unwrap();
                        let a_nid = a_emap.get(b_sid).unwrap();

                        node_pairs.push((*a_nid, *b_nid));
                    }
                },
                (RellE::Exclusive(b_sid, b_nid), RellE::Exclusive(a_sid, a_nid)) =>
                {
                    // If both are exclusive, they must go to the same symbol
                    if b_sid != a_sid
                    {
                        return Some(std::cmp::Ordering::Greater);
                    }
                    node_pairs.push((*a_nid, *b_nid));
                },
                (RellE::NonExclusive(b_emap), RellE::Exclusive(a_sid, a_nid)) =>
                {
                    // If A!C exists then B.C must exist
                    if !b_emap.contains_key(a_sid)
                    {
                        return Some(std::cmp::Ordering::Greater);
                    }

                    let b_nid = b_emap.get(a_sid).unwrap();
                    node_pairs.push((*a_nid, *b_nid));
                },
                (RellE::Empty, _) =>
                {
                    // Node is a leaf in B, Node is NOT a leaf in A - this is ok, carry on
                    continue
                },
                (_, _) =>
                {
                    return Some(std::cmp::Ordering::Greater);
                }
            }
        }
        Some(std::cmp::Ordering::Less)
    }
}

impl std::fmt::Display for RellTree
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        let mut to_visit = vec![(Self::NID_ROOT, 0, false)];

        while !to_visit.is_empty()
        {
            let (nid, depth, comes_from_exclusive) = to_visit.pop().unwrap();
            let node = self.nodes.get(&nid).unwrap();

            for i in 0..depth
            {
                let tw = if i == (depth-1) && comes_from_exclusive
                {

                    "*"
                }
                else
                {
                    "-"
                };

                write!(f, "{}", tw)?;
            }
            writeln!(f, "{}", self.symbols.get_sym(&node.sym).unwrap())?;

            match &node.edge
            {
                RellE::Exclusive(_, nid) =>
                {
                    to_visit.push((*nid, depth+1, true));
                },
                RellE::NonExclusive(map) =>
                {
                    map.values().for_each(|nid| to_visit.push((*nid, depth+1, false)));
                },
                _ => {}
            }
        }

        Ok(())
    }
}

impl Default for RellTree
{
    fn default() -> Self
    {
        Self::new()
    }
}
