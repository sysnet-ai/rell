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
        let sid = if let RellSymValue::Literal(ssym) = &sym.val
        {
            self.get_sid(ssym)
        }
        else
        {
            panic!("TODO: IMPLEMENT NUMERIC VALUES SEPARATEDLY - HOW DID YOU GET HERE!?");
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
                    (RellE::Empty, _) => {
                        // B is a leaf, in A is not this is ok
                        continue
                    },
                    (_, _) => {
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
}
