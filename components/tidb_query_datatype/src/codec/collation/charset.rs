// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

use std::str;

use super::*;

#[inline]
fn is_utf8_continuation(byte: u8) -> bool {
    matches!(byte, 0x80..=0xBF)
}

#[inline]
fn decoded_utf8_char(code: u32, len: usize) -> Option<(char, usize)> {
    // SAFETY: Callers pass only code points decoded from validated UTF-8
    // byte sequences, excluding overlong encodings, surrogates, and values
    // above U+10FFFF.
    Some((unsafe { std::char::from_u32_unchecked(code) }, len))
}

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

        let invalid = Some((std::char::REPLACEMENT_CHARACTER, 1));
        match first {
            0xC2..=0xDF => {
                let Some(&b1) = data.get(1) else {
                    return invalid;
                };
                if !is_utf8_continuation(b1) {
                    return invalid;
                }
                let code = (u32::from(first & 0x1F) << 6) | u32::from(b1 & 0x3F);
                decoded_utf8_char(code, 2)
            }
            0xE0 => {
                let Some(&b1) = data.get(1) else {
                    return invalid;
                };
                let Some(&b2) = data.get(2) else {
                    return invalid;
                };
                if !matches!(b1, 0xA0..=0xBF) || !is_utf8_continuation(b2) {
                    return invalid;
                }
                let code = (u32::from(first & 0x0F) << 12)
                    | (u32::from(b1 & 0x3F) << 6)
                    | u32::from(b2 & 0x3F);
                decoded_utf8_char(code, 3)
            }
            0xE1..=0xEC | 0xEE..=0xEF => {
                let Some(&b1) = data.get(1) else {
                    return invalid;
                };
                let Some(&b2) = data.get(2) else {
                    return invalid;
                };
                if !is_utf8_continuation(b1) || !is_utf8_continuation(b2) {
                    return invalid;
                }
                let code = (u32::from(first & 0x0F) << 12)
                    | (u32::from(b1 & 0x3F) << 6)
                    | u32::from(b2 & 0x3F);
                decoded_utf8_char(code, 3)
            }
            0xED => {
                let Some(&b1) = data.get(1) else {
                    return invalid;
                };
                let Some(&b2) = data.get(2) else {
                    return invalid;
                };
                if !matches!(b1, 0x80..=0x9F) || !is_utf8_continuation(b2) {
                    return invalid;
                }
                let code = (u32::from(first & 0x0F) << 12)
                    | (u32::from(b1 & 0x3F) << 6)
                    | u32::from(b2 & 0x3F);
                decoded_utf8_char(code, 3)
            }
            0xF0 => {
                let Some(&b1) = data.get(1) else {
                    return invalid;
                };
                let Some(&b2) = data.get(2) else {
                    return invalid;
                };
                let Some(&b3) = data.get(3) else {
                    return invalid;
                };
                if !matches!(b1, 0x90..=0xBF)
                    || !is_utf8_continuation(b2)
                    || !is_utf8_continuation(b3)
                {
                    return invalid;
                }
                let code = (u32::from(first & 0x07) << 18)
                    | (u32::from(b1 & 0x3F) << 12)
                    | (u32::from(b2 & 0x3F) << 6)
                    | u32::from(b3 & 0x3F);
                decoded_utf8_char(code, 4)
            }
            0xF1..=0xF3 => {
                let Some(&b1) = data.get(1) else {
                    return invalid;
                };
                let Some(&b2) = data.get(2) else {
                    return invalid;
                };
                let Some(&b3) = data.get(3) else {
                    return invalid;
                };
                if !is_utf8_continuation(b1)
                    || !is_utf8_continuation(b2)
                    || !is_utf8_continuation(b3)
                {
                    return invalid;
                }
                let code = (u32::from(first & 0x07) << 18)
                    | (u32::from(b1 & 0x3F) << 12)
                    | (u32::from(b2 & 0x3F) << 6)
                    | u32::from(b3 & 0x3F);
                decoded_utf8_char(code, 4)
            }
            0xF4 => {
                let Some(&b1) = data.get(1) else {
                    return invalid;
                };
                let Some(&b2) = data.get(2) else {
                    return invalid;
                };
                let Some(&b3) = data.get(3) else {
                    return invalid;
                };
                if !matches!(b1, 0x80..=0x8F)
                    || !is_utf8_continuation(b2)
                    || !is_utf8_continuation(b3)
                {
                    return invalid;
                }
                let code = (u32::from(first & 0x07) << 18)
                    | (u32::from(b1 & 0x3F) << 12)
                    | (u32::from(b2 & 0x3F) << 6)
                    | u32::from(b3 & 0x3F);
                decoded_utf8_char(code, 4)
            }
            _ => return Some((std::char::REPLACEMENT_CHARACTER, 1)),
        }
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
