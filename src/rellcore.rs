use std::collections::BTreeMap;
use std::hash::Hash;

// CORE
pub type NID = usize; // NODE ID
pub type SID = u64; // SYMBOL ID
pub const NID_INVALID: NID = 0;
pub const NID_ROOT: NID = 1;
pub trait SIDGenerator
{
    fn get_sid<S>(&self, sym:S) -> SID
        where S: AsRef<str>, S: Hash;
}

// TODO: Composable errors
pub mod errors
{
    pub type Result<T> = std::result::Result<T, Error>;
    #[derive(Debug, Clone)]
    pub enum Error
    {
        InvalidChar(char, usize),
        CustomError(String)
    }
    impl std::error::Error for Error {}

    impl std::fmt::Display for Error
    {
        fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
        {
            match self
            {
                Error::CustomError(m) => formatter.write_str(m),
                _ => formatter.write_str(&self.to_string())
            }
        }
    }
}
use errors::{Result, Error};

#[derive(Debug, PartialEq)]
pub struct RellN
{
    pub edge: RellE,
    pub sym: SID
}
impl RellN
{
    pub fn insert(&mut self, sid: &SID, nid: &NID)
    {
        self.edge.insert(sid, nid);
    }

    pub fn upgrade(&mut self, to_edge: &RellE) -> Result<()>
    {
        match (&self.edge, to_edge)
        {
            (&RellE::Empty, other) =>
            {
                self.edge = other.clone();
                Ok(())
            },
            (_, _) => Err(Error::CustomError(format!("CANT UPGRADE {:?} TO {:?}", self.edge, to_edge)))
        }
    }
}

impl std::fmt::Display for RellN
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "Node[{}]:", self.sym)?;
        write!(f, "{}", self.edge)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RellE
{
    Empty,
    NonExclusive(BTreeMap<SID, NID>),
    Exclusive(SID, NID),
}
impl std::fmt::Display for RellE
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self
        {
            Self::Empty =>
            {
            },
            Self::Exclusive(_sid, _nid) =>
            {
            },
            _ => {}
        }
        write!(f, "NODE")
    }
}
impl RellE
{
    pub fn insert(&mut self, sidref: &SID, nidref: &NID)
    {
        match self
        {
            Self::Empty => panic!("Inserting in Empty Edge suggest an issue with upstream Edge upgrading"),
            Self::NonExclusive(edge_map) => { edge_map.insert(*sidref, *nidref); }
            Self::Exclusive(sid, nid) => { *sid = *sidref; *nid = *nidref; }
        }
    }

    pub fn get<'a>(&'a self, sidref: &SID) -> Option<&'a NID>
    {
        match self
        {
            Self::Empty => None,
            Self::NonExclusive(edge_map) => { edge_map.get(sidref) },
            Self::Exclusive(sid, nid) => {
                if *sid == *sidref { Some(&nid) }
                else { None }
            }
        }
    }

    pub fn is_incompatible(&self, other: &Self) -> bool
    {
        !self.is_compatible(other)
    }

    pub fn is_compatible(&self, other: &Self) -> bool
    {
        match (self, other)
        {
            (_, Self::Empty) => true,
            (Self::NonExclusive(_), Self::NonExclusive(_)) => true,
            (Self::Exclusive(_, _), Self::Exclusive(_, _)) => true,
            _ => false,
        }
    }
}
#[derive(Debug)]
pub enum RellSymValue
{
    Numeric(f32),
    Literal(String),
}
#[derive(Debug)]
pub struct RellSym
{
    pub val: RellSymValue,
    // ref count?
}


impl std::fmt::Display for RellSym
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match &self.val
        {
            RellSymValue::Numeric(n) =>
            {
                write!(f, "{}", n)
            },
            RellSymValue::Literal(s) =>
            {
                write!(f, "{}", s)
            }
        }
    }
}
