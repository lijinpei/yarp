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
