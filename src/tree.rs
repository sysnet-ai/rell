use std::collections::BTreeMap;

use crate::rellcore::*;
use crate::rellcore::errors::*;
use crate::parser::*;

// TREE
#[derive(Debug)]
pub struct RellTree
{
    pub symbols: BTreeMap<SID, RellSym>, // SID -> Symbol Map
    pub nodes:   BTreeMap<NID, RellN>,  // NID -> Node Map
    pub next_id: NID,
}

impl std::fmt::Display for RellTree
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        let mut to_visit = vec![(NID_ROOT, 0)];

        while !to_visit.is_empty()
        {
            let (nid, depth) = to_visit.pop().unwrap();
            let node = self.nodes.get(&nid).unwrap();

            for _ in 0..depth
            {
                write!(f, "-")?;
            }
            writeln!(f, "{}", self.symbols.get(&node.sym).unwrap())?;

            match &node.edge
            {
                RellE::Exclusive(_, nid) =>
                {
                    to_visit.push((*nid, depth+1));
                },
                RellE::NonExclusive(map) =>
                {
                    map.values().for_each(|nid| to_visit.push((*nid, depth+1)));
                },
                _ => {}
            }
        }

        Ok(())
    }
}

impl RellTree
{
    pub fn new() -> Self
    {
        //
        let mut ret = Self { symbols: BTreeMap::new(), nodes: BTreeMap::new(), next_id: NID_ROOT + 1 };
        let sid = ret.get_sid("ROOT");
        ret.nodes.insert(NID_ROOT, RellN { edge: RellE::NonExclusive(BTreeMap::new()), sym: sid });
        ret.symbols.insert(sid, RellSym { val: RellSymValue::Literal("ROOT".to_string()) });
        ret
    }

    pub fn get_root<'a>(&'a self) -> &'a RellN
    {
        self.nodes.get(&NID_ROOT).unwrap()
    }

    pub fn get_mut_root<'a>(&'a mut self) -> &'a mut RellN
    {
        self.nodes.get_mut(&NID_ROOT).unwrap()
    }

    pub fn add_statement<S>(&mut self, statement: S) -> Result<Vec<NID>>
        where S: AsRef<str>
    {
        let statement = statement.as_ref();
        let (mut statement_tree, syms) = RellParser::parse_simple_statement(statement, self)?;

        let (start_at, insert_nid) = {

            let mut insert_nid = NID_ROOT;
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
                    ---p\n\
                    --q\n\
                    ---r\n");
        Ok(())
    }
}
