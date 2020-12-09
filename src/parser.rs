use std::collections::HashMap;

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
                        ParseToken::Exclusive    => RellE::Exclusive(sid, NID_INVALID),
                        ParseToken::NonExclusive => RellE::NonExclusive(HashMap::new()),
                        ParseToken::EOL => RellE::Empty,
                        err_tok => return Err(Error::CustomError(format!("Found Token {:?} after {:?}", err_tok, token))),
                    };

                    nodes.push( RellN { edge, sym: sid } );
                    syms.push ( RellSym { val: RellSymValue::Literal(sym.to_string()) } )
                }
                err_tok => {
                    return Err(Error::CustomError(format!("Upstream Error: TOKENIZER - received unreasonable token sequence TOKEN: {:?} when expecting SYMBOL(S,f,e)", err_tok)));
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

            qt.push(ParseToken::Symbol(sid, scan, i_eos));

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

    const INVALID_TOKENS: &'static str = "%$@#,][";
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

            if  RellParser::INVALID_TOKENS.contains(curr_c)
            {
                return Err(Error::InvalidChar(curr_c, str_i));
            }
        }
        Ok(statement.len())
    }
}
