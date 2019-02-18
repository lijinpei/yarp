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

#[cfg(test)]
mod tests {
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
