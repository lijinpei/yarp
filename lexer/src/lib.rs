#![feature(rustc_private, hash_raw_entry, never_type)]
#![feature(generators, generator_trait)]

extern crate syntax;

pub use syntax::parse::token::Token;
pub mod utf8;

use std::collections::*;
use std::pin::Pin;
use std::ops::Generator;


type IntKey = usize;


pub struct StringInterner {
    storage: HashMap<Box<str>, usize>,
    index: Vec<*const str>,
}

impl StringInterner {
    pub fn empty() -> StringInterner {
        StringInterner {
            storage: HashMap::new(),
            index: vec![],
        }
    }
    pub fn insert<Q: ?Sized>(&mut self, string: &Q) -> IntKey
    where
        Box<str>: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq,
        Q: AsRef<str>,
    {
        use std::collections::hash_map::*;
        match self.storage.raw_entry_mut().from_key(string) {
            RawEntryMut::Occupied(occ) => *occ.get() as IntKey,
            RawEntryMut::Vacant(vac) => {
                let ret = self.index.len();
                let (k, _) = vac
                    .insert(string.as_ref().to_string().into_boxed_str(), ret);
                self.index.push(k as &str);
                ret
            }
        }
    }

    pub fn get(&self, key: IntKey) -> &str {
        unsafe { &*self.index[key as usize] }
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }
}
/*
/// we need to internal number to make complex double's size fitts into a token
pub struct NumInterner {
    index: Vec<LitNum>,
}

impl NumInterner {
    pub fn empty() -> NumInterner {
        NumInterner { index: vec![] }
    }

    pub fn insert(&mut self, num: &LitNum) -> IntKey {
        let ret = self.index.len();
        self.index.push(*num);
        ret
    }

    pub fn get(&self, key: IntKey) -> LitNum {
        self.index[key].clone()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }
}
*/

pub trait StrProvider {
    type Handle:Copy;

    fn get(&self) -> Option<Self::Handle>;
    fn put(&self, h:Self::Handle);
    fn from_handle(&self, h:Self::Handle) -> &str;
}

pub struct Ptr<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> Ptr<'a> {
    pub fn new<'b>(s: &'b str) -> Ptr<'b> {
        Ptr {
            s,
            pos: 0usize,
        }
    }

    pub fn ret(&self) -> usize {
        return self.pos;
    }

    pub fn capacity_left(&self) -> usize {
        self.s.len() - self.pos
    }

    pub fn bump(&mut self) -> Option<char> {
        utf8::parse_utf8(&self.s, &mut self.pos)
    }
}

pub struct Lexer {
    pub string_interner: StringInterner,
//    pub num_interner: NumInterner,
}


impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            string_interner: StringInterner::empty(),
//            num_interner: NumInterner::empty(),
        }
    }

    pub fn calc(&mut self) -> i32 {
        return 1;
    }

    pub fn lex<'a, T:StrProvider>(mut self: Pin<&'a mut Self>, provider: T) -> impl Generator + 'a {
        return move || {
            loop {
                yield Some(self.calc());
            }
            return 0usize;
        };
        /*
        let len = input.len();
        if input.len() < 4 {
            return (Token::Eof(len), 0);
        }
        let mut ptr = Ptr::new(input);
        match ptr.bump() {
            Some(c) => {
                match (c) {
                    '=' => {
                        return self.start_eq(&mut ptr);
                    },
                    '<' => {
                        return self.start_le(&mut ptr);
                    },
                    '>' => {
                        return self.start_ge(&mut ptr);
                    },
                    '&' => {
                        return self.start_and(&mut ptr);
                    },
                    '|' => {
                        return self.start_or(&mut ptr);
                    },
                    '!' => {
                        return self.start_not(&mut ptr);
                    },
                    '~' => {
                        return self.start_tilde(&mut ptr);
                    },
                    '@' => {
                        return self.start_at(&mut ptr);
                    },
                    '.' => {
                        return self.start_dot(&mut ptr);
                    },
                    ',' => {
                        return self.start_comma(&mut ptr);
                    },
                    ';' => {
                        return self.start_semi(&mut ptr);
                    },
                    ':' => {
                        return self.start_colon(&mut ptr);
                    },
                    '#' => {
                        return self.start_pound(&mut ptr);
                    },
                    '$' => {
                        return self.start_dollar(&mut ptr);
                    },
                    '?' => {
                        return self.start_question(&mut ptr);
                    },
                    '\'' => {
                        return self.start_quote(&mut ptr);
                    },
                    '{' => {
                        return self.start_open_brace(&mut ptr);
                    },
                    '}' => {
                        return self.start_closing_brace(&mut ptr);
                    },
                    '(' => {
                        return self.start_open_paren(&mut ptr);
                    },
                    ')' => {
                        return self.start_closing_paren(&mut ptr);
                    },
                    '[' => {
                        return self.start_open_bracket(&mut ptr);
                    },
                    ']' => {
                        return self.start_closing_bracket(&mut ptr);
                    },
                    '+' => {
                        return self.start_add(&mut ptr);
                    },
                    '-' => {
                        return self.start_minus(&mut ptr);
                    },
                    '*' => {
                        return self.start_star(&mut ptr);
                    },
                    '/' => {
                        return self.start_slash(&mut ptr);
                    },
                    '%' => {
                        return self.start_percentage(&mut ptr);
                    },
                    '^' => {
                        return self.start_caret(&mut ptr);
                    },
                    _ => (),
                }
            },
            None => {
                return (Token::Error{}, ptr.ret());
            },
        }
        panic!();
        */
    }
}

impl Lexer {
    fn start_eq(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_le(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_ge(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_and(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_or(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_not(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_tilde(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_at(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_dot(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_comma(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_semi(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_colon(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_pound(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_dollar(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_question(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_quote(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_open_brace(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_closing_brace(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_open_paren(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_closing_paren(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_open_bracket(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_closing_bracket(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_add(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_minus(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_star(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_slash(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_percentage(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }

    fn start_caret(&mut self, ptr: &mut Ptr<'_>) -> (Token, usize) {
        panic!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn prop_1(ss: Vec<String>) -> bool {
        let mut interner = StringInterner::empty();
        let mut index = vec![];
        for s in ss.iter() {
            index.push(interner.insert(s.as_str()));
        }
        for (i, s) in ss.iter().enumerate() {
            if interner.get(index[i]) != s {
                return false;
            }
        }
        for (i, s) in ss.iter().enumerate() {
            interner.insert(s.as_str());
            if interner.get(index[i]) != s {
                return false;
            }
        }
        return true;
    }

    #[quickcheck]
    fn check_1(ss: Vec<String>) -> bool {
        prop_1(ss)
    }

    #[quickcheck]
    fn check_2(ss: Vec<Vec<u8>>) -> bool {
        prop_1(
            ss.iter()
                .map(|v| unsafe {
                    std::str::from_utf8_unchecked(v.as_slice()).to_string()
                })
                .collect(),
        )
    }
}
