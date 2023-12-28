use std::collections::BTreeMap;
use std::collections::btree_map::Iter;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::rellcore::*;

#[derive(Debug, Default, PartialEq)]
pub struct SymbolsTable
{
    pub symbols: BTreeMap<SID, RellSym>,
    bound_variables: BTreeMap<SID, SID>
}
impl SymbolsTable
{
    pub fn new() -> Self { Self::default() }

    pub fn get_sym(&self, sid: &SID) -> Option<&RellSym>
    {
        if let Some(bound_sid) = self.bound_variables.get(sid)
        {
            self.symbols.get(bound_sid)
        }
        else
        {
            self.symbols.get(sid)
        }
    }

    pub fn get_sym_val(&self, sid: &SID) -> &RellSymValue
    {
        self.get_sym(sid).unwrap().get_val()
    }

    pub fn get_sid_no_binding<S>(&self, sym: S) -> SID 
        where S: AsRef<str>, S: Hash
    {
        let mut hasher = DefaultHasher::new();
        sym.hash(&mut hasher);
        let v = hasher.finish();
        v
    }

    pub fn insert(&mut self, key: SID, value: RellSym) -> Option<RellSym>
    {
        // If insert finds a value already present with that key, it returns
        // it as part of the insertion
        self.symbols.insert(key, value)
    }

    pub fn symbols_iter(&self) -> Iter<SID, RellSym>
    {
        self.symbols.iter()
    }

    pub fn bind_variables(&mut self, variable_values: &mut BTreeMap<SID, SID>)
    {
        debug!("Binding Starting on {:?} with {:?}", self.symbols, variable_values);
        variable_values.iter().for_each(|(_k_sid, v_sid)|
        {
            /*
            TODO: This breaks because we don't register the literals as we parse them in the
            binding code. I am unsure if this was a design decision or fix or deliberate or just an
            oversight, but this is starting to point in the direction of the whole symbols table needing some serious love.
            match self.symbols.get(k_sid)
            {
                None => { panic!("Binding non existing symbol {}", k_sid) },
                Some(s) => {
                    if let RellSymValue::Identifier(_) = s.get_val()
                    {
                        // all good
                    }
                    else
                    {
                        panic!("Trying to bind literal symbol {:?} ", s.get_val());
                    }
                }
            }
            */

            if self.symbols.get(v_sid).is_none()
            {
                panic!("Trying to bind to unexisting symbol");
            }
        });

        self.bound_variables.append(variable_values);
    }

    pub fn clear_bindings(&mut self)
    {
        self.bound_variables = BTreeMap::new();
    }
}

impl SIDGenerator for SymbolsTable
{
    fn get_sid<S>(&self, sym:S) -> SID
        where S: AsRef<str>, S: Hash
    {
        let v = self.get_sid_no_binding(sym);

        if let Some(other_sid) = self.bound_variables.get(&v)
        {
            *other_sid
        }
        else
        {
            v
        }
    }
}
