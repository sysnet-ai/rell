use crate::rellcore::*;
use crate::rellcore::errors::*;
use crate::parser::*;
use crate::tree::*;
use std::collections::BTreeMap;

#[derive(Debug)]
struct BindingVarState
{
    nid: NID,
    path: String,
    bound_vars: Vec<(SID, SID)>,
}

#[derive(Default)]
pub struct BindingState
{
    binding_statements: BTreeMap<String, Option<Vec<BindingVarState>>>, // Pre-Bound Statement -> BindingState
}
impl BindingState
{
    pub fn new() -> Self { Self::default() }

    pub fn add_statement<S>(&mut self, statement: S) -> &mut Self
        where S: AsRef<str>
    {
        let statement = statement.as_ref().to_string();
        self.binding_statements.entry(statement).or_insert(None);
        self
    }

    pub fn bind_all(&mut self, tree: &RellTree)
    {
        let mut new_bs = BTreeMap::new();
        for statement in self.binding_statements.keys()
        {
            new_bs.insert(statement.clone(), Some(
                self.bind_statement_to_tree(statement, tree).unwrap()
            ));
        }
        self.binding_statements = new_bs;
    }

    pub fn generate_compatible(&self) -> Vec<BTreeMap<SID, SID>>
    {
        let mut valid_dictionaries = vec![BTreeMap::new()];

        for binding_states in self.binding_statements.values()
        {
            let mut new_valid_dicts = vec![];
            while !valid_dictionaries.is_empty() && binding_states.is_some()
            {
                let cur_dic = valid_dictionaries.pop().unwrap();
                let mut compatible;

                for bs in binding_states.as_ref().unwrap()
                {
                    let mut cur_dic = cur_dic.clone();
                    compatible = true;

                    for (b_var_name, b_var_val) in &bs.bound_vars
                    {
                        compatible = match cur_dic.get(b_var_name)
                        {
                            Some(sid) => {
                                sid == b_var_val
                            },
                            None => {
                               cur_dic.insert(*b_var_name, *b_var_val);
                               true
                            }
                        };

                        if !compatible
                        {
                            break;
                        }
                    }
                    if compatible
                    {
                        new_valid_dicts.push(cur_dic);
                    }
                }
            }
            valid_dictionaries = new_valid_dicts;
        }

        valid_dictionaries
    }

    fn bind_statement_to_tree<S>(&self, statement: S, tree: &RellTree) -> Result<Vec<BindingVarState>>
        where S: AsRef<str>
    {
        let (_, parsed_symbols) = RellParser::parse_simple_statement(statement, &tree.symbols)?;
        self.bind_parsed_statement_to_tree(tree, &parsed_symbols)
    }

    fn bind_parsed_statement_to_tree(&self, tree: &RellTree, stmnt_symbols: &[RellSym]) -> Result<Vec<BindingVarState>>
    {
        let mut var_states_to_visit = vec![BindingVarState { nid:  RellTree::NID_ROOT,
                                                             path: "".to_string(),
                                                             bound_vars: vec![] }];
        for sym in stmnt_symbols
        {
            let mut new_ntv = vec![];
            while !var_states_to_visit.is_empty()
            {
                let cur_n = var_states_to_visit.pop().unwrap();
                let node = tree.nodes.get(&cur_n.nid).unwrap();
                if let RellSymValue::Identifier(id) = sym.get_val()
                {
                    let nids = match &node.edge {
                        RellE::Exclusive(_, a_nid) => vec![*a_nid],
                        RellE::NonExclusive(a_map) => a_map.values().cloned().collect(),
                        _                          =>  vec![]
                    };

                    for nid in nids
                    {
                        Self::binding_traversal_helper(&nid, tree, &cur_n.bound_vars, &cur_n.path, &mut new_ntv, Some(&tree.symbols.get_sid(id)))
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
                                Self::binding_traversal_helper(&a_nid, tree, &cur_n.bound_vars, &cur_n.path, &mut new_ntv, None);
                            }
                        },
                        RellE::NonExclusive(a_map) =>
                        {
                            if let Some(a_nid) = a_map.get(&sym_id)
                            {
                                Self::binding_traversal_helper(&a_nid, tree, &cur_n.bound_vars, &cur_n.path, &mut new_ntv, None);
                            }
                        },
                        _ => {}
                    }
                }
            }
            var_states_to_visit = new_ntv;
        }
        Ok(var_states_to_visit)
    }

    fn binding_traversal_helper(nid: &NID, tree: &RellTree, bound_vars: &[(SID, SID)], path: &str, new_ntv: &mut Vec<BindingVarState>, id_opt: Option<&SID>)
    {
        let mut bound_vars = bound_vars.to_owned();
        let nnode = tree.nodes.get(&nid).unwrap();
        let nsym = tree.symbols.get_sym(&nnode.sym).unwrap();
        let new_path = path.to_owned() + &nsym.to_string() + &nnode.edge.to_string();
        if let Some(id)  = id_opt
        {
            bound_vars.push((*id, nnode.sym));
        }
        new_ntv.push(BindingVarState { nid: *nid, path: new_path, bound_vars });
    }
}

#[cfg(test)]
mod test
{
    fn build_test_tree() -> Result<RellTree>
    {
        let mut w = RellTree::new();
        w.add_statement("city.in.state")?;
        w.add_statement("state.in.country")?;
        w.add_statement("other_state.in.country")?;
        w.add_statement("nothing.important")?;
        w.add_statement("something.in")?;

        Ok(w)
    }

    use super::*;
    #[test]
    fn test_binding() -> Result<()>
    {
        let w = build_test_tree()?;

        let bs = BindingState::new();
        let b_result_1 = bs.bind_statement_to_tree("X.in.Y", &w)?;
        let b_result_2 = bs.bind_statement_to_tree("Y.in.Z", &w)?;

        let expected_result_1 = vec![vec![("X", "state"), ("Y", "country")],
                                     vec![("X", "city"),  ("Y", "state")],
                                     vec![("X", "other_state"), ("Y", "country")]];
        let expected_result_procd_1:Vec<Vec<(SID, SID)>> = expected_result_1.iter().map(|vars|
            vars.iter().map( |(var_n, var_v)| { (w.symbols.get_sid(var_n), w.symbols.get_sid(var_v)) } ).collect()
        ).collect();

        let b_result_1_procd: Vec<Vec<(SID, SID)>> = b_result_1.iter().map(|bres| {
            bres.bound_vars.iter().map(|(bvar_n, bvar_v)| (*bvar_n, *bvar_v)).collect()
        }).collect();
        assert_eq!(expected_result_procd_1, b_result_1_procd, "Incorrect result for binding to tree");

        let expected_result_2 = vec![vec![("Y", "state"), ("Z", "country")],
                                     vec![("Y", "city"),  ("Z", "state")],
                                     vec![("Y", "other_state"), ("Z", "country")]];
        let b_result_2_procd: Vec<Vec<(SID, SID)>> = b_result_2.iter().map(|bres| {
            bres.bound_vars.iter().map(|(bvar_n, bvar_v)| (*bvar_n, *bvar_v)).collect()
        }).collect();
        let expected_result_procd_2:Vec<Vec<(SID, SID)>> = expected_result_2.iter().map(|vars|
            vars.iter().map( |(var_n, var_v)| { (w.symbols.get_sid(var_n), w.symbols.get_sid(var_v)) } ).collect()
        ).collect();
        assert_eq!(expected_result_procd_2, b_result_2_procd, "Incorrect result for binding to tree");

        Ok(())
    }

    #[test]
    fn test_binding_state() -> Result<()>
    {
        let mut w = build_test_tree()?;
        let mut bs = BindingState::new();
        bs.add_statement("X.in.Y");
        bs.add_statement("Y.in.Z");

        let x_sid = w.symbols.get_sid("X");
        let y_sid = w.symbols.get_sid("Y");
        let z_sid = w.symbols.get_sid("Z");

        bs.bind_all(&w);

        let mut compatible_var_bindings = bs.generate_compatible();

        assert_eq!(compatible_var_bindings.len(), 1, "Incorrect length for bindings result");
        assert_eq!(*compatible_var_bindings[0].get(&x_sid).unwrap(), w.symbols.get_sid("city"), "Incorrect value for binding" );
        assert_eq!(*compatible_var_bindings[0].get(&y_sid).unwrap(), w.symbols.get_sid("state"), "Incorrect value for binding" );
        assert_eq!(*compatible_var_bindings[0].get(&z_sid).unwrap(), w.symbols.get_sid("country"), "Incorrect value for binding" );

        w.symbols.bind_variables(&mut compatible_var_bindings[0]);

        assert_eq!(w.symbols.get_sym(&x_sid).unwrap().to_string(), "city");
        assert_eq!(w.symbols.get_sym(&y_sid).unwrap().to_string(), "state");
        assert_eq!(w.symbols.get_sym(&z_sid).unwrap().to_string(), "country");

        assert_eq!(w.get_at_path("city.in.Y").unwrap().sym, w.symbols.get_sid("state"), "Incorrect substitution after variable binding");

        w.symbols.clear_bindings();
        assert!(w.get_at_path("city.in.Y").is_none(), "Incorrect result after binding clearing");
        assert!(w.get_at_path("city.in.state").is_some(), "Posterior not triggering after clearing binding");

        Ok(())
    }

}
