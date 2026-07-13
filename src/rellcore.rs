use std::collections::BTreeMap;
use std::hash::Hash;

pub enum EdgePairKind<'a>
{
    BothEmpty,
    OneEmpty
    {
        live: &'a RellE,
    },
    BothNonExclusive
    {
        a_map: &'a BTreeMap<SID, NID>,
        b_map: &'a BTreeMap<SID, NID>,
    },
    BothExclusiveSameSid
    {
        sid: SID,
        a_nid: NID,
        b_nid: NID,
    },
    ExclusiveNonExclusive
    {
        sid: SID,
        x_nid: NID,
        nex_nid: NID,
    },
    Incompatible,
}

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

    pub fn get<'a>(&'a self, sidref: &SID) -> Option<&'a NID>
    {
        self.edge.get(&sidref)
    }

    pub fn insert(&mut self, sid: &SID, nid: &NID)
    {
        self.edge.insert(sid, nid);
    }

    pub fn remove(&mut self, sid: &SID) -> NID
    {
        self.edge.remove(sid)
    }

    pub fn upgrade(&mut self, to_edge: &RellE)
    {
        match (&self.edge, to_edge)
        {
            (&RellE::Empty, other) =>
            {
                self.edge = other.clone();
            },
            (_, _) => panic!("Cant Upgrade {:?} TO {:?}", self.edge, to_edge)
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
    pub fn classify_pair<'a>(a: &'a RellE, b: &'a RellE) -> EdgePairKind<'a>
    {
        match (a, b)
        {
            (RellE::Empty, RellE::Empty) => EdgePairKind::BothEmpty,
            (RellE::Empty, _) => EdgePairKind::OneEmpty { live: b },
            (_, RellE::Empty) => EdgePairKind::OneEmpty { live: a },
            (RellE::NonExclusive(am), RellE::NonExclusive(bm)) =>
                EdgePairKind::BothNonExclusive { a_map: am, b_map: bm },
            (RellE::Exclusive(a_sid, a_nid), RellE::Exclusive(b_sid, b_nid)) =>
            {
                if a_sid == b_sid
                {
                    EdgePairKind::BothExclusiveSameSid { sid: *a_sid, a_nid: *a_nid, b_nid: *b_nid }
                }
                else
                {
                    EdgePairKind::Incompatible
                }
            },
            (RellE::Exclusive(x_sid, x_nid), RellE::NonExclusive(nex)) |
            (RellE::NonExclusive(nex), RellE::Exclusive(x_sid, x_nid)) =>
            {
                if nex.len() == 1 && nex.contains_key(x_sid)
                {
                    EdgePairKind::ExclusiveNonExclusive {
                        sid: *x_sid, x_nid: *x_nid, nex_nid: *nex.get(x_sid).unwrap(),
                    }
                }
                else { EdgePairKind::Incompatible }
            },
        }
    }

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

    pub fn remove(&mut self, sid: &SID) -> NID
    {
        match self {
            Self::Empty => { panic!("Removing from Empty Edge") },
            Self::Exclusive(s, n) => {
                let n = *n;
                if s != sid
                {
                   panic!("Removing a non exisiting connection from Exclusive Edge signals an issue upstream");
                }
                else
                {
                    *self = Self::Empty;
                    n
                }
            },
            Self::NonExclusive(connections) => {
                if let Some(nid) = connections.remove(&sid)
                {
                    nid
                }
                else
                {
                    panic!("Removing non existing connection from NonExclusive Edge signals an issue upstream"); 
                }
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
            Self::Empty =>           { "" },
            Self::Exclusive(_, _) => { "!" },
            Self::NonExclusive(_) => { "." }
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


#[cfg(test)]
mod test
{
}
