#![cfg_attr(feature = "cargo-clippy", allow(clippy::suspicious_else_formatting))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub mod rellcore;
use rellcore::*;

pub mod parser;

pub mod tree;
use tree::*;

pub mod tree_traits;

impl SIDGenerator for RellTree
{
    fn get_sid<S>(&self, sym:S) -> SID
        where S: AsRef<str>, S: Hash
    {
        let mut hasher = DefaultHasher::new();
        sym.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::parser::*;
    use crate::rellcore::errors::*;

    #[test]
    fn scanner_test()
    {
        assert_eq!(RellParser::find_next_eos("brown.is!happy", 0).unwrap(), 5);
        assert_eq!(RellParser::find_next_eos("brown.is!happy", 6).unwrap(), 8);
        assert_eq!(RellParser::find_next_eos("brown.is!happy", 9).unwrap(), 14);
        assert_eq!(RellParser::find_next_eos("brown.is,", 0).unwrap(), 5);

        // Error cases
        {
            let err = RellParser::find_next_eos("brown.is,happy", 6);
            assert!(if let Err(Error::InvalidChar(',', 8)) = err { true } else { false }, "Result is: {:?}", err);
        }

        {
            let err = RellParser::find_next_eos("brown@is.happy", 0);
            assert!(if let Err(Error::InvalidChar('@', 5)) = err { true } else { false }, "Result is: {:?}", err);
        }

        {
            let err = RellParser::find_next_eos("brown.is.", 6);
            assert!(if let Err(Error::InvalidChar('.', 8)) = err { true } else { false }, "Result is: {:?}", err);
        }
    }

    #[test]
    fn tokenization()
    {
        let w = RellTree::new();
        let expected = vec![ParseToken::Symbol(w.get_sid("brown"), 0, 5), ParseToken::Exclusive,
                            ParseToken::Symbol(w.get_sid("is"),    6, 8), ParseToken::EOL];

        assert_eq!(expected, RellParser::tokenize("brown!is", &w).unwrap());
    }

    #[test]
    fn parse() -> Result<()>
    {
        let w = RellTree::new();
        let err = RellParser::parse_simple_statement("brown..nope", &w);
        assert!(if let Err(Error::InvalidChar('.', 6)) = err { true } else { false }, "Result is: {:?}", err);

        let (_, syms) = RellParser::parse_simple_statement("brown.lastname.perez", &w)?;

        let expected = vec![
            RellSym { val: RellSymValue::Literal("brown".to_string())    },
            RellSym { val: RellSymValue::Literal("lastname".to_string()) },
            RellSym { val: RellSymValue::Literal("perez".to_string())    }];
        assert_eq!(syms, expected, "{:?}", syms);

        let (_, syms2) = RellParser::parse_simple_statement("brown.height!50", &w)?;
        let expected2 = vec![
            RellSym { val: RellSymValue::Literal("brown".to_string())    },
            RellSym { val: RellSymValue::Literal("height".to_string()) },
            RellSym { val: RellSymValue::Numeric(50.0)    }];
        assert_eq!(syms2, expected2, "{:?}", syms2);

        let result = RellParser::parse_simple_statement("brown.height!5m", &w);
        if let Err(Error::CustomError(_)) = result
        {
            Ok(())
        }
        else
        {
            Err(Error::CustomError(format!("Unexpected Result {:?}", result)))
        }
    }


    #[test]
    fn t_start() -> Result<()>
    {
        let mut w = RellTree::new();
        w.add_statement("brown.is!happy")?;
        w.add_statement("brown.knows.stuff")?;
        w.add_statement("brown.knows.me")?;

        assert_eq!(w.add_statement("brown.knows").unwrap(), vec![]); // Already know all of this info, nothing to insert
        assert_eq!(w.add_statement("brown.is").unwrap(), vec![]);

        w.add_statement("brown.is!sad")?;
        let node_id_of_brownissadtoday = w.add_statement("brown.is!sad.today").unwrap()[0];
        assert_eq!(w.query("brown.is!sad.today").unwrap(), w.nodes.get(&node_id_of_brownissadtoday).unwrap());
        assert_eq!(w.query("brown.is.sad.today").unwrap(), w.nodes.get(&node_id_of_brownissadtoday).unwrap()); // !sad satifies .sad

        assert!(w.query("brown.is!happy.today").is_none()); // !happy cant be satisfied by !sad
        assert!(w.query("brown!is!sad.today").is_none()); // !is can't be satisfied by .is

        let e = w.add_statement("brown.is.sad.today");
        if let Err(Error::CustomError(_)) = e
        {
            Ok(())
        }
        else
        {
            Err(Error::CustomError(format!("Unexpected Result {:?}", e)))
        }
    }
}
