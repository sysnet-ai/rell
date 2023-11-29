use std::collections::BTreeMap;

use crate::rellcore::*;
use crate::rellcore::errors::{ Result, Error };

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ParseToken
{
    Symbol(SID, usize, usize),
    Exclusive,
    NonExclusive,
    EOL
}
pub struct RellParser;
impl RellParser
{
    pub const NID_INVALID: NID = 0;
    pub fn parse_simple_statement<S, SF>(statement: S, sidfactory: &SF) -> Result<(Vec<RellN>, Vec<RellSym>)> // Simple Statments in EL: A | A.B | A!B
        where S: AsRef<str>, SF: SIDGenerator
    {

        let mut nodes = vec![];
        let mut syms = vec![];
        let statement = statement.as_ref();
        let tokens = Self::tokenize(statement, sidfactory)?;

        let mut token_it = tokens.iter().peekable();
        while token_it.peek().is_some()
        {
            let token = token_it.next().unwrap();
            match token
            {
                ParseToken::Symbol(sid, from, to) => {
                    let sym = statement.get(*from..*to).unwrap();
                    let sid = *sid;

                    let edge = match token_it.next().unwrap()
                    {
                        ParseToken::Exclusive    => RellE::Exclusive(sid, Self::NID_INVALID),
                        ParseToken::NonExclusive => RellE::NonExclusive(BTreeMap::new()),
                        ParseToken::EOL => RellE::Empty,
                        err_tok => return Err(Error::CustomError(format!("Found Token {:?} after {:?}", err_tok, token))),
                    };

                    let n = RellN { edge, sym: sid, parent: RellN::NID_INVALID };
                    println!("{:?}", n);
                    nodes.push(n);

                    let val = match sym.chars().next().unwrap()
                    {
                        '0'..='9' => {
                            match sym.parse::<f32>()
                            {
                                Ok(num) => RellSymValue::Numeric(num),
                                Err(e)  => return Err(Error::CustomError(format!("{}", e))),
                            }
                        },
                        'A'..='Z' => {
                            RellSymValue::Identifier(sym.to_string())
                        },

                        // No need to check for invalid characters, that was done by the tokenizer
                        _ => { RellSymValue::Literal(sym.to_string()) }
                    };
                    println!("{:?}",val);
                    syms.push(RellSym::new(val))
                },
                err_tok => {
                    return Err(Error::CustomError(format!("Upstream Error: TOKENIZER - received unreasonable token sequence TOKEN: {:?} when expecting a SYMBOL", err_tok)));
                }
            }
        }

        Ok((nodes, syms))
    }

    pub fn tokenize<S, SF>(statement: S, sidfactory: &SF) -> Result<Vec<ParseToken>>
        where S: AsRef<str>, SF: SIDGenerator
    {
        let statement = statement.as_ref();
        let mut qt = vec![];

        let mut scan = 0;
        while scan < statement.len()
        {
            let i_eos = Self::find_next_eos(statement, scan)?;
            let sym = statement.get(scan..i_eos).unwrap();
            let sid = sidfactory.get_sid(&sym);

            let pt = ParseToken::Symbol(sid, scan, i_eos);
            println!("TOKEN: {:?}", pt);
            qt.push(pt);

            qt.push(
                match statement.get(i_eos..i_eos+1)
                {
                    Some(".") => ParseToken::NonExclusive,
                    Some("!") => ParseToken::Exclusive,
                    None      => ParseToken::EOL,
                    Some(ch) => return Err(Error::CustomError(format!("Upstream Error: SCANNER - marked {} as EOS at {}", ch, i_eos))),
                }
            );
            scan = i_eos+1;
        }

        Ok(qt)
    }

    const INVALID_CHARS: &'static str = "%$@#,][";
    pub fn find_next_eos<S>(statement: S, start: usize) -> Result<usize>
        where S: AsRef<str>
    {
        let statement = statement.as_ref();

        for (i, curr_c) in statement.chars().skip(start).enumerate()
        {
            let str_i = start + i;
            if let '.' | '!' = curr_c
            {
                if  i == 0 || str_i == statement.len() - 1 // ! or . at the beginning or end is not allowed
                {
                    return Err(Error::InvalidChar(curr_c, str_i));
                }
                return Ok(str_i);
            }

            if  RellParser::INVALID_CHARS.contains(curr_c)
            {
                return Err(Error::InvalidChar(curr_c, str_i));
            }
        }
        Ok(statement.len())
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use crate::symbols::*;
    
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
        let w = SymbolsTable::new();
        let expected = vec![ParseToken::Symbol(w.get_sid("brown"), 0, 5), ParseToken::Exclusive,
                            ParseToken::Symbol(w.get_sid("is"),    6, 8), ParseToken::EOL];

        assert_eq!(expected, RellParser::tokenize("brown!is", &w).unwrap());
    }

    #[test]
    fn parse() -> Result<()>
    {
        let w = SymbolsTable::new();
        let err = RellParser::parse_simple_statement("brown..nope", &w);
        assert!(if let Err(Error::InvalidChar('.', 6)) = err { true } else { false }, "Result is: {:?}", err);

        let (_, syms) = RellParser::parse_simple_statement("brown.lastname.perez", &w)?;

        let expected = vec![
            RellSym::new( RellSymValue::Literal("brown".to_string())    ),
            RellSym::new( RellSymValue::Literal("lastname".to_string()) ),
            RellSym::new( RellSymValue::Literal("perez".to_string())    )];
        assert_eq!(syms, expected, "{:?}", syms);

        let (_, syms2) = RellParser::parse_simple_statement("brown.height!50", &w)?;
        let expected2 = vec![
            RellSym::new( RellSymValue::Literal("brown".to_string()) ),
            RellSym::new( RellSymValue::Literal("height".to_string())),
            RellSym::new( RellSymValue::Numeric(50.0)    )];
        assert_eq!(syms2, expected2, "{:?}", syms2);

        let (_, syms3) = RellParser::parse_simple_statement("brown.Height!50", &w)?;
        let expected3 = vec![
            RellSym::new( RellSymValue::Literal("brown".to_string())    ),
            RellSym::new( RellSymValue::Identifier("Height".to_string()) ),
            RellSym::new( RellSymValue::Numeric(50.0)    )];
        assert_eq!(syms3, expected3, "{:?}", syms3);


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
}


