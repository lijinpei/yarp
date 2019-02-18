pub mod token;

use token::LitNum;

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
