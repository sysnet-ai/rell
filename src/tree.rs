use std::collections::BTreeMap;

use crate::rellcore::*;
use crate::rellcore::errors::*;
use crate::parser::*;

// TREE
#[derive(Debug, PartialEq)]
pub struct RellTree
{
    pub symbols: BTreeMap<SID, RellSym>, // SID -> Symbol Map
    pub nodes:   BTreeMap<NID, RellN>,  // NID -> Node Map
    pub next_id: NID,
}

impl RellTree
{
    const NID_ROOT: NID = 1;
    pub fn new() -> Self
    {
        //
        let mut ret = Self { symbols: BTreeMap::new(), nodes: BTreeMap::new(), next_id: Self::NID_ROOT + 1 };
        let sid = ret.get_sid("ROOT");
        ret.nodes.insert(Self::NID_ROOT, RellN { edge: RellE::NonExclusive(BTreeMap::new()), sym: sid });
        ret.symbols.insert(sid, RellSym { val: RellSymValue::Literal("ROOT".to_string()) });
        ret
    }

    pub fn get_root<'a>(&'a self) -> &'a RellN
    {
        self.nodes.get(&Self::NID_ROOT).unwrap()
    }

    pub fn get_mut_root<'a>(&'a mut self) -> &'a mut RellN
    {
        self.nodes.get_mut(&Self::NID_ROOT).unwrap()
    }

    pub fn add_statement<S>(&mut self, statement: S) -> Result<Vec<NID>>
        where S: AsRef<str>
    {
        let statement = statement.as_ref();
        let (mut statement_tree, syms) = RellParser::parse_simple_statement(statement, self)?;

        let (start_at, insert_nid) = {

            let mut insert_nid = Self::NID_ROOT;
            let mut start_at   = statement.len();

            let mut r = self.get_mut_root();

            for (i, node) in statement_tree.iter().enumerate()
            {
                if let Some(nid) = r.edge.get(&node.sym)
                {
                    insert_nid = *nid;
                    r = self.nodes.get_mut(&insert_nid).unwrap();

                    if r.edge.is_incompatible(&node.edge)
                    {
                        r.upgrade(&node.edge)?;
                    }
                }
                else
                {
                    start_at = i;
                    break;
                }
            }
            (start_at, insert_nid)
        };

        if start_at == statement.len()
        {
            return Ok(vec![]); // Everything in that statement is already known
        }

        let new_nids:Vec<NID> = statement_tree.iter().skip(start_at).map(|_| self.get_next_nid()).collect();

        let mut new_r = self.nodes.get_mut(&insert_nid).unwrap();
        for (i, node) in statement_tree.iter_mut().skip(start_at).enumerate()
        {
            let new_nid = new_nids[i];
            new_r.insert(&node.sym, &new_nid);
            new_r = node;
        }

        for (i, node) in statement_tree.drain(start_at..).enumerate()
        {
            self.nodes.insert(new_nids[i], node);
        }

        for s in syms
        {
            self.add_symbol_instance(s);
        }

        Ok(new_nids)
    }

    pub fn query<'a, S>(&'a self, statement: S) -> Option<&'a RellN>
        where S: AsRef<str>
    {
        let statement = statement.as_ref();
        let parsed_query = RellParser::tokenize(statement, self);

        if let Ok(query_tokens) = parsed_query
        {
            let mut r = self.get_root();
            for t in query_tokens
            {
                match (t, &r.edge)
                {
                    (ParseToken::Symbol(sid, _, _), edge) => if let Some(nid) = edge.get(&sid)
                    {
                        r = self.nodes.get(&nid).unwrap();
                    }
                    else
                    {
                        return None;
                    },
                    (ParseToken::Exclusive, RellE::NonExclusive(_)) => { return None; },
                    (_,_) => {},
                }
            }
            return Some(r);
        }
        None
    }

    fn add_symbol_instance(&mut self, sym: RellSym)
    {
        let sid = match &sym.val
        {
            RellSymValue::Literal(ssym) =>
            {
                self.get_sid(ssym)
            },
            RellSymValue::Numeric(_ssym) =>
            {
                panic!("Numeric Symbols aren't fully implemented right now");
            },
        };

        self.symbols.insert(sid, sym);
    }

    fn get_next_nid(&mut self) -> NID
    {
        // NOT THREAD SAFE!! - hehe
        let v = self.next_id;
        self.next_id += 1;
        v
    }
}

// GLB and LUB operations
impl RellTree
{
    // Greatest Lower Bound - Union of Trees
    pub fn greatest_lower_bound(&self, other: &Self) -> Option<Self>
    {
        let mut glb = RellTree::new();

        let mut node_trios = vec![(RellTree::NID_ROOT, RellTree::NID_ROOT, RellTree::NID_ROOT)];
        while !node_trios.is_empty()
        {
            let (a_nid, b_nid, glb_nid) = node_trios.pop().unwrap();
            let a_node   = self.nodes.get(&a_nid).unwrap();
            let b_node   = other.nodes.get(&b_nid).unwrap();

            match(&a_node.edge, &b_node.edge)
            {
                (RellE::NonExclusive(a_emap), RellE::NonExclusive(b_emap)) =>
                {
                    // If both non-exclusive add to GLB node the union of
                    // both maps and add new nodes appropriately
                    //

                    for (&sym, a_nid) in a_emap
                    {

                        if let Some(b_nid) = b_emap.get(&sym)
                        {
                            // Symbol exists in both nodes, insert into glb and add them to the
                            // queue
                            let new_nid = glb.insert_into(&glb_nid, RellN { edge: RellE::Empty, sym }, false).unwrap();
                            node_trios.push((*a_nid, *b_nid, new_nid))
                        }
                        else
                        {
                            glb.clone_subgraph_into(&glb_nid, self, a_nid, false).unwrap();
                        }
                    }

                    for (&sym, b_nid) in b_emap
                    {
                        if a_emap.get(&sym).is_none()
                        {
                            // Symbol exists only in B, copy subtree here
                            glb.clone_subgraph_into(&glb_nid, other, b_nid, false).unwrap();
                        }
                        // else {...} already taken care of in the loop above
                    }
                },
                (RellE::Empty, e) | (e, RellE::Empty) =>
                {
                    // If one side is empty, just clone the other tree into the GLB-tree
                    match e 
                    {
                        RellE::Exclusive(_, x_nid) => {
                            glb.clone_subgraph_into(&glb_nid, other, x_nid, true).unwrap();
                        },
                        RellE::NonExclusive(nex_map) => {
                            for (_, nex_nid) in nex_map
                            {
                                glb.clone_subgraph_into(&glb_nid, other, nex_nid, false).unwrap();
                            }
                        },
                        _ => {}
                    }
                },
                (RellE::Exclusive(a_sid, a_nid), RellE::Exclusive(b_sid, b_nid)) =>
                {
                    // If they both go to the same symbol add, else incompat
                    if a_sid == b_sid
                    {
                        let new_nid = glb.insert_into(&glb_nid, RellN { edge: RellE::Empty, sym: *a_sid }, true).unwrap();
                        node_trios.push((*a_nid, *b_nid, new_nid));
                    }
                    else
                    {
                        return None; // Incompatible Trees
                    }
                },
                (RellE::Exclusive(x_sid, x_nid), RellE::NonExclusive(nex_map)) | (RellE::NonExclusive(nex_map), RellE::Exclusive(x_sid, x_nid)) =>
                {
                    // if an X and a NX edges are found, they're considered compatible IFF they go
                    // to the same symbol AND the NX edge goes to no other symbols
                    //
                    if !nex_map.contains_key(&x_sid) || nex_map.len() != 1
                    {
                        return None;
                    }

                    let new_nid = glb.insert_into(&glb_nid, RellN { edge: RellE::Empty, sym: *x_sid }, true).unwrap();
                    node_trios.push((*x_nid, *nex_map.get(&x_sid).unwrap(), new_nid));
                }
            }
        }

        // Add all symbols from both trees into the GLB
        for sym_tbl in vec![&self.symbols, &other.symbols]
        {
            //TODO: This just stomps all values, Ref counting?
            for (sid, sym) in sym_tbl
            {
                glb.symbols.insert(*sid, sym.clone());
            }
        }

        Some(glb)
    }


    // TODO: I don't like these 2 functions
    fn insert_into(&mut self, into_n: &NID, new_node: RellN, exclusive: bool) -> Result<NID>
    {
        //TODO: Not really taking care of repeated edges, shouldn't affect the data but
        // wastes memory
        let new_nid = self.get_next_nid();
        let sid = new_node.sym;
        let insert_node = self.nodes.get_mut(into_n).unwrap();

        if exclusive
        {
            insert_node.upgrade(&RellE::Exclusive(sid, new_nid))?;
        }
        else
        {
            match insert_node.edge
            {
                RellE::Empty => { insert_node.upgrade(&RellE::NonExclusive(BTreeMap::new()))?; },
                _ => { /*...?*/ },
            }

            insert_node.insert(&sid, &new_nid);
        }
        self.nodes.insert(new_nid,  new_node);

        Ok(new_nid)
    }

    fn clone_subgraph_into(&mut self, into_n: &NID, other_tree: &RellTree, from_n: &NID, exclusive: bool) -> Result<NID>
    {
        let sym = other_tree.nodes.get(from_n).unwrap().sym;

        let new_subgraph_root = RellN { edge: RellE::Empty, sym };
        let new_subgraph_nid = self.insert_into(into_n, new_subgraph_root, exclusive)?;

        let mut node_pairs = vec![(new_subgraph_nid, *from_n)];
        while !node_pairs.is_empty()
        {
            let (self_nid, c_nid) = node_pairs.pop().unwrap();

            let c_node = other_tree.nodes.get(&c_nid).unwrap();

            match &c_node.edge
            {
                &RellE::Exclusive(c_sid, c_nid) =>
                {
                    let new_node = RellN { edge: RellE::Empty, sym: c_sid };
                    let new_nid = self.insert_into(&self_nid, new_node, true)?;
                    node_pairs.push((new_nid, c_nid));
                },
                RellE::NonExclusive(c_map) =>
                {
                   for (&c_sid, &c_nid) in c_map
                    {
                        let new_node = RellN { edge: RellE::Empty, sym: c_sid };
                        let new_nid = self.insert_into(&self_nid, new_node, false)?;
                        node_pairs.push((new_nid, c_nid));
                    }
                },
                _ =>
                {
                }
            }
        }
        Ok(new_subgraph_nid)
    }
}

mod traitimpls
{
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
                        // If both are exclusive, they must have go to the same symbol
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
                        // B is a leaf, in A is not this is ok
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
                writeln!(f, "{}", self.symbols.get(&node.sym).unwrap())?;

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
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn test_fmt() -> Result<()>
    {
        let mut t = RellTree::new();
        t.add_statement("a.b.c")?;
        t.add_statement("a.b.d")?;
        t.add_statement("a.f.e")?;
        t.add_statement("z.q.r")?;
        t.add_statement("z.x!p")?;

        assert!(t.add_statement("z!y!y").is_err()); // Incompatible with z.x!p

        assert_eq!(
            format!("{}", t),
                   "ROOT\n\
                    -a\n\
                    --b\n\
                    ---d\n\
                    ---c\n\
                    --f\n\
                    ---e\n\
                    -z\n\
                    --x\n\
                    --*p\n\
                    --q\n\
                    ---r\n");
        Ok(())
    }

    #[test]
    fn test_cmp() -> Result<()>
    {
        let mut t = RellTree::new();
        t.add_statement("a.b.c")?;
        let mut t2 = RellTree::new();
        let mut t3 = RellTree::new();

        t2.add_statement("a")?;
        assert!(t < t2, "{} < {}", t, t2);

        t2.add_statement("a.b")?;
        assert!(t < t2, "{} < {}", t, t2);

        // This makes t2 > t
        t2.add_statement("a.c")?;
        assert!(t > t2, "{} > {}", t, t2);

        // Verify adding new statements
        // wont break cmp
        t2.add_statement("a.b.c")?;
        assert!(t > t2, "{} > {}", t, t2);

        t3.add_statement("a.b.c")?;
        assert!(t < t3, "{} < {}", t, t3);

        Ok(())
    }

    #[test]
    fn test_cmp2() -> Result<()>
    {
        let mut t = RellTree::new();
        t.add_statement("t!a")?;
        let mut t2 = RellTree::new();
        t2.add_statement("t.a")?;

        assert!(t < t2, "{} < {}", t, t2);
        assert!(t2 > t, "{} > {}", t2, t);

        t = RellTree::new();
        t.add_statement("t.a")?;
        t.add_statement("t.b")?;

        t2 = RellTree::new();
        t2.add_statement("t.a")?;

        assert!(t < t2, "{} < {}", t, t2);
        assert!(t2 > t, "{} > {}", t2, t);

        t = RellTree::new();
        t.add_statement("t.a")?;
        t2 = RellTree::new();
        t2.add_statement("t.a.b")?;

        assert!(t > t2, "{} < {}", t, t2);

        Ok(())
    }

    #[test]
    fn test_cmp3() -> Result<()>
    {
        let mut t = RellTree::new();
        t.add_statement("t.a.b")?;
        t.add_statement("t.b.c")?;
        t.add_statement("t.c!d.e")?;

        let mut t2 = RellTree::new();
        t2.add_statement("t.b")?;
        t2.add_statement("t.a")?;
        t2.add_statement("t.c.d")?;
        assert!(t < t2, "{} < {}", t, t2);

        t2.add_statement("t.c.d.e")?;
        assert!(t < t2, "{} < {}", t, t2);

        Ok(())
    }

    #[test]
    fn test_clone_into() -> Result<()>
    {
        Ok(())
    }

    #[test]
    fn test_glb() -> Result<()>
    {
        let mut t = RellTree::new();
        t.add_statement("t.a")?;
        let mut t2 = RellTree::new();
        t2.add_statement("t.b")?;

        let glb = t.greatest_lower_bound(&t2).unwrap();
        
        assert_eq!(format!("{}", glb), 
                          "ROOT\n\
                           -t\n\
                           --b\n\
                           --a\n");
        //
        let mut t3 = RellTree::new();
        let mut t4 = RellTree::new();

        t3.add_statement("t!a.b")?;
        t4.add_statement("t!a.c.d")?;
        let glb2 = t3.greatest_lower_bound(&t4).unwrap();
        assert_eq!(format!("{}", glb2), 
                          "ROOT\n\
                           -t\n\
                           -*a\n\
                           ---b\n\
                           ---c\n\
                           ----d\n");
        
        let mut t5 = RellTree::new();
        let mut t6 = RellTree::new();
        t5.add_statement("t!a")?;
        t6.add_statement("t.b")?;

        let glb3 = t5.greatest_lower_bound(&t6);

        assert!(glb3.is_none(), "{}", glb3.unwrap());

        Ok(())
    }
}
