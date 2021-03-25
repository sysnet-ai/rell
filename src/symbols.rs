use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::rellcore::*;
use crate::rellcore::errors::*;

#[derive(Debug, PartialEq)]
pub struct SymbolsTable
{
    symbols: BTreeMap<SID, RellSym> 
}

impl SymbolsTable
{
    pub fn new() -> Self
    {
        Self { symbols: BTreeMap::new() }
    }

    pub fn get_sym_table(&self) -> &BTreeMap<SID, RellSym>
    {
        &self.symbols
    }

    pub fn get_sym_table_mut(&mut self) -> &mut BTreeMap<SID, RellSym>
    {
        &mut self.symbols
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