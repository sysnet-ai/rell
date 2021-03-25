use std::collections::BTreeMap;

use crate::rellcore::*;
use crate::rellcore::errors::*;
use crate::parser::*;
use crate::tree::*;
use crate::symbols::*;


struct BindingState<'a>
{
    variables: BTreeMap<String, BindingVar>, // Identifier -> Bound Variable State
    nodes: BTreeMap<String, &'a RellN>
}
impl<'a> BindingState<'a>
{
    pub fn new() -> Self
    {
        Self { variables: BTreeMap::new(), nodes: BTreeMap::new() }
    }

    pub fn add_bindable_statement<S>(&mut self, statement: S) -> Result<()>
      where S: AsRef<str>
    {

        let mut sym_table = SymbolsTable::new();
        let (statement_tree, syms) = RellParser::parse_simple_statement(statement, &sym_table)?; //TODO: Need to redo the SIDFactory trait

        let mut path_to_node = "".to_string();
        for (s_inx, sym) in syms.iter().enumerate()
        {
            let is_var = if let RellSymValue::Identifier(_) = sym.get_val() { true } else { false };

            if is_var
            {
                let exclusive = &path_to_node[path_to_node.len()-1..] == "!"; 
                let var = BindingVar {
                                       bound_value: None,
                                       path_to_parent: path_to_node[..path_to_node.len()-1].to_string(), // Remove the last char
                                       nonex_iter_count: 0,
                                       exclusive,
                                     }; 
                println!("{:?} {}", var, sym.get_display());
                self.variables.insert(sym.get_display(), var);
            }

            path_to_node += &sym.get_display();
            path_to_node += match statement_tree[s_inx].edge
            {
                RellE::NonExclusive(_) => ".",
                RellE::Exclusive(_, _) => "!",
                RellE::Empty => "",
            };
        }
        Ok(())
    }

    pub fn init_binding_on_tree(&mut self, tree: &'a RellTree)
    {
        for (var_name, var_state) in self.variables.iter_mut()
        {
            if var_state.is_unbound()
            {
                if let Some(node) = var_state.bind_to(tree)
                {
                    self.nodes.insert(var_name.to_string(), node);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct BindingVar
{
    bound_value: Option<RellSymValue>,
    path_to_parent: String,
    nonex_iter_count: usize,
    exclusive: bool,
}
impl BindingVar
{
    pub fn bind_to<'a>(&mut self, tree: &'a RellTree) -> Option<&'a RellN>
    {
        self.bound_value = None;
        if let Some(q_result) = tree.query(&self.path_to_parent)
        {
            if self.bind(q_result, tree)
            {
                return Some(q_result);
            }
        }
        None
    }

    pub fn bind(&mut self, node: &RellN, tree: &RellTree) -> bool
    {
        match (&node.edge, self.exclusive)
        {
            (RellE::Exclusive(sid, _nid), true) => 
            {
                self.bound_value = Some(tree.symbols.get_sym_table().get(&sid).unwrap().get_val().clone());
                true
            },
            (RellE::NonExclusive(map), false) =>
            {
                let sid_opt = map.keys().skip(self.nonex_iter_count).next();

                if let Some(sid) = sid_opt
                {
                    self.bound_value = Some(tree.symbols.get_sym_table().get(&sid).unwrap().get_val().clone());
                    self.nonex_iter_count += 1;
                };

                sid_opt.is_some()
            },
            (_, _) =>
            {
                false
            }
        }
    }

    pub fn is_bound(&self) -> bool
    {
        self.bound_value.is_some()
    }

    pub fn is_unbound(&self) -> bool
    {
        !self.is_bound()      
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
        bs.add_bindable_statement("city.in.X")?;

        let bs2 = BindingState::new();
        bs.add_bindable_statement("city.in.X!Y")?;

        let mut w = RellTree::new();
        w.add_statement("city.in.state")?;
        w.add_statement("city.in.country")?;

        bs.init_binding_on_tree(&w);
        println!("{:?}", bs.variables);

        //TODO: This are just a stub

        Ok(())
    }
}
