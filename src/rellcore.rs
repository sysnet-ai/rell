use std::collections::BTreeMap;
use std::hash::Hash;

// CORE
pub type NID = usize; // NODE ID   (Monotonically increased from 1)
pub type SID = u64;   // SYMBOL ID (Hashed from value)

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
                Error::InvalidChar(ch, pos)  => formatter.write_fmt(format_args!("Invalid Char {} at {}", ch, pos))
            }
        }
    }
}
use errors::{Result, Error};

#[derive(Debug, PartialEq)]
pub struct RellN
{
    pub edge: RellE,
    pub sym: SID,
    pub parent: NID,
}
impl RellN
{
    pub const NID_INVALID: NID = 0;

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

#[derive(Debug, Clone, PartialEq)]
pub enum RellE
{
    Empty,
    NonExclusive(BTreeMap<SID, NID>),
    Exclusive(SID, NID),
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
        matches!((self, other), (_, Self::Empty) |
                                (Self::NonExclusive(_), Self::NonExclusive(_)) |
                                (Self::Exclusive(_,_), Self::Exclusive(_,_)))
    }
}
impl std::fmt::Display for RellE
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        let s = match self
        {
            Self::Empty =>           { ' ' },
            Self::Exclusive(_, _) => { '!' },
            Self::NonExclusive(_) => { '.' }
        };

        write!(formatter, "{}", s)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RellSymValue
{
    Numeric(f32),
    Literal(String),
    Identifier(String)
}

#[derive(Debug, PartialEq, Clone)]
pub struct RellSym
{
    val: RellSymValue,
}
impl RellSym
{
    pub fn new(val: RellSymValue) -> Self
    {
        Self { val }
    }

    fn get_display(&self) -> String
    {
        match &self.get_val()
        {
            RellSymValue::Numeric(n) =>
            {
                n.to_string()
            },
            RellSymValue::Literal(s) | RellSymValue::Identifier(s) =>
            {
                s.to_string()
            }
        }
    }

    pub fn get_val(&self) -> &RellSymValue
    {
        &self.val
    }
}

impl std::fmt::Display for RellSym
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        write!(f, "{}", self.get_display())
    } 
}
