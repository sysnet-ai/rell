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
    symbols: BTreeMap<SID, RellSym>,
}
impl SymbolsTable
{
    pub fn new() -> Self { Self::default() }

    pub fn get_sym(&self, sid: &SID) -> Option<&RellSym>
    {
        self.symbols.get(sid)
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
}

impl SIDGenerator for SymbolsTable
{
    fn get_sid<S>(&self, sym:S) -> SID
        where S: AsRef<str>, S: Hash
    {
        let mut hasher = DefaultHasher::new();
        sym.hash(&mut hasher);
        hasher.finish()
    }
}