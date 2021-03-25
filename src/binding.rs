use std::collections::BTreeMap;

use crate::rellcore::*;
use crate::rellcore::errors::*;
use crate::parser::*;
use crate::tree::*;
use crate::symbols::*;


struct BindingState
{
    statements: Vec<(Vec<RellN>, Vec<RellSym>)>, // List of pre-parsed binding statements
}
#[derive(Debug)]
struct BindingVarState
{
    nid: NID,
    path: String,
    bound_vars: Vec<(String, RellSym)>
}
impl BindingState
{
    pub fn new() -> Self
    {
        Self { statements: vec![] }
    }

    pub fn add_bindable_statement<S>(&mut self, statement: S) -> Result<()>
        where S: AsRef<str>
    {
        let sym_table = SymbolsTable::new();
        let parse_result = RellParser::parse_simple_statement(statement, &sym_table)?;
        self.statements.push(parse_result);
        Ok(())
    }

    pub fn init_binding_from_tree(&mut self, tree: &RellTree)
    {
        for (_, stmnt_symbols) in &self.statements
        {
            self.init_biniding_from_statement(tree, stmnt_symbols);
        }
    }

    pub fn init_biniding_from_statement(&self, tree: &RellTree, stmnt_symbols: &Vec<RellSym>) -> Vec<BindingVarState>
    {
        let mut nodes_to_verify = vec![BindingVarState { nid: RellTree::NID_ROOT, path: "".to_string(), bound_vars: vec![] }];
        for sym in stmnt_symbols
        {
            let mut new_ntv = vec![];
            while !nodes_to_verify.is_empty()
            {
                let cur_n = nodes_to_verify.pop().unwrap();
                let node = tree.nodes.get(&cur_n.nid).unwrap();
                if let RellSymValue::Identifier(id) = sym.get_val()
                {
                    let nids = match &node.edge
                    {
                        RellE::Exclusive(_, a_nid) => {
                            vec![*a_nid]
                        },
                        RellE::NonExclusive(a_map) => {
                            a_map.values().cloned().collect()
                        },
                        _ => { vec![] }
                    };

                    for nid in nids
                    {
                        let mut bound_vars = cur_n.bound_vars.clone();
                        let nnode = tree.nodes.get(&nid).unwrap();
                        let new_path = cur_n.path.clone() + &tree.symbols.get_sym(&nnode.sym).unwrap().to_string() + &nnode.edge.to_string();
                        bound_vars.push((id.clone(), tree.symbols.get_sym(&nnode.sym).unwrap().clone()));
                        new_ntv.push(BindingVarState { nid, path: new_path.clone(), bound_vars });
                    }
                }
                else
                {
                    let sym_id = tree.symbols.get_sid(&sym.to_string());
                    match &node.edge
                    {
                        RellE::Exclusive(a_sid, a_nid) =>
                            {
                                if *a_sid == sym_id
                                {
                                    let nnode = tree.nodes.get(&a_nid).unwrap();
                                    let new_path = cur_n.path.clone() + &tree.symbols.get_sym(&nnode.sym).unwrap().to_string() + &nnode.edge.to_string();
                                    new_ntv.push(BindingVarState { nid: *a_nid, path: new_path.clone(), bound_vars: cur_n.bound_vars.clone() });
                                }
                            },
                        RellE::NonExclusive(a_map) =>
                            {
                                if let Some(a_nid) = a_map.get(&sym_id)
                                {
                                    let nnode = tree.nodes.get(&a_nid).unwrap();
                                    let new_path = cur_n.path.clone() + &tree.symbols.get_sym(&nnode.sym).unwrap().to_string() + &node.edge.to_string();
                                    new_ntv.push(BindingVarState { nid: *a_nid, path: new_path.clone(), bound_vars: cur_n.bound_vars.clone() });
                                }
                            },
                        _ => {}
                    }
                }
            }
            println!("{:?}", new_ntv);
            nodes_to_verify = new_ntv;
        }
        nodes_to_verify
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    #[test]
    fn test_binding() -> Result<()>
    {
        let mut bs = BindingState::new();
        bs.add_bindable_statement("X.in.Y")?;

        // let bs2 = BindingState::new();
        // bs.add_bindable_statement("city.in.X!Y")?;

        let mut w = RellTree::new();
        w.add_statement("city.in.state")?;
        w.add_statement("state.in.country")?;
        w.add_statement("other_state.in.country")?;
        w.add_statement("nothing.important")?;
        w.add_statement("something.in")?;

        bs.init_binding_from_tree(&w);

        //TODO: Write a real test :)
        panic!("the disco!");

        Ok(())
    }
}
