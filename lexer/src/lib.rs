#![feature(rustc_private, hash_raw_entry, never_type)]
#![feature(generators, generator_trait)]
#![feature(trait_alias)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

extern crate syntax;

pub use syntax::parse::token::Token;
pub mod utf8;

use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

pub enum Utf8Error {
    InvalidLeading,
    InvalidCont,
    InvalidScalarValue,
}

pub enum CharResult {
    Ok(char),
    NeedMoreU8,
}

pub enum TokenResult {
    Ok(Token),
    NeedMoreU8,
    NeedMoreChar,
}

// an endless u8 generator
pub trait U8Generator<'a> = Generator<Yield = &'a [u8], Return = !>;
// an endless char generator, unless meets invalid utf8, or don't have enought u8 to decode utf8 char
pub trait CharGenerator = Generator<Yield = CharResult, Return = Utf8Error>;
// and endless Token generator, unless unlerlying CharGenerator didn't, and don't see enough char to decide on a whole token(eg. '=' vs '==')
pub trait TokenGenerator = Generator<Yield = TokenResult, Return = Utf8Error>;

pub type IntKey = usize;
/*
pub struct StringInterner {
    storage: HashMap<Box<str>, usize>,
    index: Vec<*const str>,
}
*/

pub fn char_generator_from_byte<
    'b,
    T: 'b + U8Generator<'b> + std::marker::Unpin,
>(
    mut source: T,
) -> impl CharGenerator + 'b
where
{
    return move || {
        let mut len: usize = 0;
        let mut pos: usize = 0;
        let mut input: &[u8] = &[];

        macro_rules! replenish {
            () => {{
                {
                    loop {
                        match Pin::new(&mut source).resume() {
                            GeneratorState::Yielded(buf) => {
                                input = buf;
                                len = buf.len();
                                if len == 0 {
                                    yield CharResult::NeedMoreU8;
                                    continue;
                                }
                                pos = 0;
                                break;
                            }
                            _ => panic!(),
                        }
                    }
                }
            }};
        }

        macro_rules! next_u8 {
            () => {
                if pos < len {
                    let p1 = pos;
                    pos += 1;
                    input[p1] as u32
                } else {
                    yield CharResult::NeedMoreU8;
                    replenish!();
                    pos = 1;
                    input[0] as u32
                }
            };
        }

        macro_rules! next_u8_cont {
            () => {{
                let c = next_u8!();
                let mask = 0b1100_0000;
                if c & mask != 0b1000_0000 {
                    return Utf8Error::InvalidCont;
                }
                c & !mask
            }};
        }

        macro_rules! ok_char {
            ($u_32:expr) => {
                unsafe { CharResult::Ok(std::char::from_u32_unchecked($u_32)) }
            };
        }

        replenish!();
        pos = 0;

        loop {
            let c1 = next_u8!();
            if c1 < 0b1000_0000u32 {
                yield ok_char!(c1);
                continue;
            }
            if c1 < 0b1100_0000u32 {
                return Utf8Error::InvalidLeading;
            }
            let c2 = next_u8_cont!();
            if c1 < 0b1110_0000u32 {
                yield ok_char!((c1 & 0x1F) << 6 | c2);
                continue;
            }
            let c3 = next_u8_cont!();
            if c1 < 0b1111_0000u32 {
                yield ok_char!((c1 & 0xF) << 12 | c2 << 6 | c3);
                continue;
            }
            let c4 = next_u8_cont!();
            yield ok_char!((c1 & 0x7) << 18 | c2 << 12 | c3 << 6 | c4);
        }
    };
}

pub fn str_to_char_slice(input: &[u8]) -> Result<Vec<char>, Utf8Error> {
    let u8_gen = || {
        yield input;
        panic!();
    };
    let mut char_gen = char_generator_from_byte(u8_gen);
    let mut ret = Vec::new();
    loop {
        match Pin::new(&mut char_gen).resume() {
            GeneratorState::Yielded(res) => match res {
                CharResult::Ok(ch) => {
                    ret.push(ch);
                }
                CharResult::NeedMoreU8 => {
                    return Ok(ret);
                }
            },
            GeneratorState::Complete(err) => {
                return Err(err);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn prop1(input: &str) -> bool {
        let vec1: Vec<char> = input.chars().collect();
        match str_to_char_slice(input.as_bytes()) {
            Ok(vec) => {
                return vec == vec1;
            }
            _ => {
                return false;
            }
        }
    }
    #[quickcheck]
    fn check_1(input: String) -> bool {
        let res = prop1(input.as_str());
        return res;
    }
}

pub fn token_generator_from_char<T: CharGenerator + std::marker::Unpin>(
    mut source: T,
) -> impl TokenGenerator {
    return move || {
        let handle_eq = handle_eq_impl(Pin::new(&mut source));
//        let handle_plus = handle_plus_impl(Pin::new(&mut source));

        macro_rules! next_char {
            () => {
                loop {
                    match Pin::new(&mut source).resume() {
                        GeneratorState::Yielded(cr) => {
                            match cr {
                                CharResult::Ok(c) => break c,
                                CharResult::NeedMoreU8 => yield TokenResult::NeedMoreU8,
                            }
                        },
                        GeneratorState::Complete(err) => {
                            return err;
                        }
                    }
                }
            }
        }

        macro_rules! my_co_await {
            ($func:ident($($exp:expr),*)) => {
                match $func(Pin::new(&mut source, $($expr),*)) {
                    GeneratorState::Yielded(tr) => yield tr,
                    GeneratorState::Complete(err) => return err,
                }
            }
        }

        loop {
            let c = next_char!();
            match c {
                '=' => {
                    //my_co_await(lex_eq());
        return Utf8Error::InvalidLeading;
                },
                _ => {
        return Utf8Error::InvalidLeading;
                }
            }
        }

        return Utf8Error::InvalidLeading;
    };
}

fn handle_eq_impl<'a, T: CharGenerator + std::marker::Unpin>(
    source: Pin<&'a mut T> 
) -> impl TokenGenerator + 'a {
    return || {
        source.resume();
        yield TokenResult::NeedMoreU8;
        return Utf8Error::InvalidLeading;
    };
}

fn handle_plus_impl<'a, T: CharGenerator + std::marker::Unpin>(
    source: Pin<&'a mut CharGenerator>
) -> impl TokenGenerator + 'a {
    return || {
        let source = source;
        source.resume();
        yield TokenResult::NeedMoreU8;
        return Utf8Error::InvalidLeading;
    };
}
/*
impl Future for CharStream {
    type Output = char;
    fn poll(self: Pin<&mut Self>, lw: &std::task::LocalWaker) -> Poll<char> {
        yield Poll::Ready('a');
        yield Poll::Pending;
        return ();
    }
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
*/

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

/*
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
fn start_eq<'a, T:StrProvider>(mut self: Pin<&'a mut Self>, provider: &T) -> impl Generator + 'a {
move || {
loop {
let c = co_await!(provider.peek());
if c != '=' {
yield Token::Eq;
}
co_await!(provider.consume());
yield Token::Eq;
}
panic!();
}
}

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
*/
