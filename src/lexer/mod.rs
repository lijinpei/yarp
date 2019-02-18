pub mod token;

use token::*;

type IntKey = usize;

use std::collections::*;

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
        parse_utf8(&self.s, &mut self.pos)
    }
}

pub struct Lexer {
    pub string_interner: StringInterner,
    pub num_interner: NumInterner,
}

pub fn parse_utf8(input: &str, start: &mut usize) -> Option<char> {
    use std::char::from_u32_unchecked;
    let c1: u8 = 0b10000000;
    let c2: u8 = 0b11000000;
    let c3: u8 = 0b11100000;
    let c4: u8 = 0b11110000;
    let c5: u8 = 0b11111000;
    let s1: u8 = 6;
    let s2: u8 = 12;
    let s3: u8 = 18;

    let is_cont = |c: u8| c & c2 == c1;

    fn is_unicode_scalar_value(i: u32) -> bool {
        return i <= 0xD7FFu32 || (i >= 0xE000u32 && i <= 0x10FFFFu32);
    }

    let s = *start;
    let i1: u8 = input.as_bytes()[s];
    let t1 = i1 & c2;
    if t1 < c1 {
        if is_unicode_scalar_value(i1 as u32) {
            *start += 1;
            return Some(i1 as char);
        }
    }
    if t1 == c1 {
        return None;
    }
    let i2: u8 = input.as_bytes()[s + 1];
    if !is_cont(i2) {
        return None;
    }
    if i1 < c3 {
        let v1: u8 = i1 & !c3;
        if v1 < 0b10u8 {
            return None;
        }
        let ret = ((v1 as u32) << s1) | ((i2 & !c2) as u32);
        if is_unicode_scalar_value(ret) {
            *start += 2;
            return Some(unsafe { from_u32_unchecked(ret) });
        } else {
            return None;
        }
    }
    let i3: u8 = input.as_bytes()[s + 2];
    if !is_cont(i3) {
        return None;
    }
    if i1 < c4 {
        let v1: u8 = i1 & !c4;
        let v2: u8 = i2 & !c2;
        if v1 == 0 && v2 < 0b100000u8 {
            return None;
        }
        let ret =
            ((v1 as u32) << s2) | ((v2 as u32) << s1) | ((i3 & !c2) as u32);
        if is_unicode_scalar_value(ret) {
            *start += 3;
            return Some(unsafe { from_u32_unchecked(ret) });
        } else {
            return None;
        }
    }
    let i4: u8 = input.as_bytes()[s + 3];
    if !is_cont(i4) || i1 >= c5 {
        return None;
    }
    let v1: u8 = i1 & !c5;
    let v2: u8 = i2 & !c2;
    if v1 == 0 && v2 < 0b10000u8 {
        return None;
    }
    let ret = ((v1 as u32) << s3)
        | ((v2 as u32) << s2)
        | (((i3 & !c2) as u32) << s1)
        | ((i4 & !c2) as u32);
    if is_unicode_scalar_value(ret) {
        *start += 4;
        return Some(unsafe { from_u32_unchecked(ret) });
    }
    return None;
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            string_interner: StringInterner::empty(),
            num_interner: NumInterner::empty(),
        }
    }

    pub fn lex<'a>(&mut self, input: &'_ str) -> (Token, usize) {
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

    fn parse_utf8_ok(i: u32) {
        // NOTE: this is plat form dependent
        let bytes: [u8; 4] = unsafe { std::mem::transmute(i.to_le()) };
        let mut r3: usize = 0;
        let s = unsafe { std::str::from_utf8_unchecked(&bytes[..]) };
        match parse_utf8(s, &mut r3) {
            None => {
                let r1 = std::str::from_utf8(&bytes[..]);
                assert!(r1.is_err());
                assert_eq!(0, r3);
            }
            Some(c) => {
                let r1 = std::str::from_utf8(&bytes[0..r3]);
                let r1 = r1.unwrap().chars().next();
                assert_eq!(r1, Some(c));
            }
        }
    }

    #[test]
    fn test_parse_utf8() {
        let mut count = 0;
        for i in 0u64..=(u32::max_value() as u64) {
            if count == 10000000 {
                count = 0;
                eprintln!("{}", i);
            }
            count += 1;
            parse_utf8_ok(i as u32);
        }
    }
}
