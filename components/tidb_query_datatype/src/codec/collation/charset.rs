// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

use std::str;

use super::*;

pub struct CharsetBinary;

impl Charset for CharsetBinary {
    type Char = u8;

    #[inline]
    fn validate(_: &[u8]) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn decode_one(data: &[u8]) -> Option<(Self::Char, usize)> {
        if data.is_empty() {
            None
        } else {
            Some((data[0], 1))
        }
    }

    fn charset() -> crate::Charset {
        crate::Charset::Binary
    }
}

pub struct CharsetUtf8mb4;

impl Charset for CharsetUtf8mb4 {
    type Char = char;

    #[inline]
    fn validate(bstr: &[u8]) -> Result<()> {
        str::from_utf8(bstr)?;
        Ok(())
    }

    #[inline]
    fn decode_one(data: &[u8]) -> Option<(Self::Char, usize)> {
        let first = *data.first()?;
        if first.is_ascii() {
            return Some((first as char, 1));
        }

        for len in 2..=4 {
            if let Some(prefix) = data.get(..len) {
                if let Ok(s) = str::from_utf8(prefix) {
                    let mut chars = s.chars();
                    if let Some(ch) = chars.next() {
                        if chars.next().is_none() {
                            return Some((ch, len));
                        }
                    }
                }
            }
        }
        Some((std::char::REPLACEMENT_CHARACTER, 1))
    }

    fn charset() -> crate::Charset {
        crate::Charset::Utf8Mb4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8mb4_decode_one_invalid_bytes() {
        assert_eq!(CharsetUtf8mb4::decode_one(&[]), None);
        assert_eq!(CharsetUtf8mb4::decode_one(b"a"), Some(('a', 1)));
        assert_eq!(
            CharsetUtf8mb4::decode_one(&[0xE4, 0xBD, 0xA0]),
            Some(('\u{4F60}', 3))
        );

        for input in [
            &[0x80][..],
            &[0xE0][..],
            &[0xE0, 0x80][..],
            &[0xF0, 0x9F][..],
            &[0xF5][..],
        ] {
            assert_eq!(
                CharsetUtf8mb4::decode_one(input),
                Some((std::char::REPLACEMENT_CHARACTER, 1)),
                "input={input:?}"
            );
        }
    }
}

// gbk character data actually stored with utf8mb4 character encoding.
pub type CharsetGbk = CharsetUtf8mb4;

// gb18030 character data actually stored with utf8mb4 character encoding.
pub type CharsetGb18030 = CharsetUtf8mb4;
