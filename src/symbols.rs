use std::collections::BTreeMap;
use std::collections::btree_map::Iter;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::rellcore::*;

#[derive(Debug, PartialEq)]
struct BindingState;

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
        //TODO: Verify that actually only Identifiers are being bound
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
        let mut hasher = DefaultHasher::new();
        sym.hash(&mut hasher);
        let v = hasher.finish();

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
