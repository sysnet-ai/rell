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

            match RellE::classify_pair(&a_node.edge, &b_node.edge)
            {
                EdgePairKind::BothEmpty => {},
                EdgePairKind::OneEmpty { .. } =>
                {
                    if !matches!(b_node.edge, RellE::Empty)
                    {
                        // A is the leaf but B has edges — A is missing info B has
                        return Some(std::cmp::Ordering::Greater);
                    }
                    // else B is the leaf — A may have more info, that's fine
                },
                EdgePairKind::BothNonExclusive { a_map, b_map } =>
                {
                    for (b_sid, b_child) in b_map
                    {
                        match a_map.get(b_sid)
                        {
                            None          => return Some(std::cmp::Ordering::Greater),
                            Some(a_child) => node_pairs.push((*a_child, *b_child)),
                        }
                    }
                },
                EdgePairKind::BothExclusiveSameSid { a_nid, b_nid, .. } =>
                {
                    node_pairs.push((a_nid, b_nid));
                },
                EdgePairKind::ExclusiveNonExclusive { x_nid, nex_nid, .. } =>
                {
                    if matches!(a_node.edge, RellE::Exclusive(_, _))
                    {
                        // A is exclusive, B is a matching non-exclusive singleton — A satisfies B
                        node_pairs.push((x_nid, nex_nid));
                    }
                    else
                    {
                        // B is exclusive, A is non-exclusive — A cannot satisfy B's stricter constraint
                        return Some(std::cmp::Ordering::Greater);
                    }
                },
                EdgePairKind::Incompatible =>
                {
                    return Some(std::cmp::Ordering::Greater);
                },
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
