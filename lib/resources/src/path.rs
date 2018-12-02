/*!

Resource path implementation.

Universal resource path help to query resources the same way across different platforms and backends.

*/

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ResourcePathBuf {
    inner: String,
}

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ResourcePath {
    inner: str,
}

impl ResourcePath {
    fn from_inner(inner: &str) -> &ResourcePath {
        unsafe { &*(inner as *const str as *const ResourcePath) }
    }
}

impl ::std::ops::Deref for ResourcePathBuf {
    type Target = ResourcePath;

    fn deref(&self) -> &ResourcePath {
        &ResourcePath::from_inner(&self.inner[..])
    }
}

impl AsRef<ResourcePath> for str {
    fn as_ref(&self) -> &ResourcePath {
        &ResourcePath::from_inner(self)
    }
}

impl AsRef<ResourcePath> for String {
    fn as_ref(&self) -> &ResourcePath {
        &ResourcePath::from_inner(&self)
    }
}

impl AsRef<ResourcePath> for ResourcePathBuf {
    fn as_ref(&self) -> &ResourcePath {
        &ResourcePath::from_inner(&self.inner)
    }
}

impl<'a> From<&'a ResourcePath> for ResourcePathBuf {
    fn from(other: &ResourcePath) -> Self {
        ResourcePathBuf {
            inner: other.inner.into(),
        }
    }
}

impl<'a> From<&'a str> for &'a ResourcePath {
    fn from(other: &'a str) -> Self {
        &ResourcePath::from_inner(other)
    }
}

impl From<String> for ResourcePathBuf {
    fn from(other: String) -> Self {
        ResourcePathBuf { inner: other }
    }
}

impl ::std::borrow::Borrow<ResourcePath> for ResourcePathBuf {
    fn borrow(&self) -> &ResourcePath {
        &ResourcePath::from_inner(&self.inner)
    }
}

impl AsRef<ResourcePath> for ResourcePath {
    fn as_ref(&self) -> &ResourcePath {
        self
    }
}

// ---- IMPL ----

impl ResourcePath {
    pub fn parent(&self) -> Option<&ResourcePath> {
        match self.inner.rfind('/') {
            Some(index) => Some(ResourcePath::from_inner(&self.inner[..index])),
            None => if &self.inner == "" {
                None
            } else {
                Some(ResourcePath::from_inner(""))
            },
        }
    }

    pub fn to_string(&self) -> String {
        self.inner.into()
    }

    pub fn items(&self) -> impl Iterator<Item = &str> {
        self.inner.split('/')
    }

    /// Returns path as str and ensures that the returned str does not have a leading or trailing slash
    pub fn as_clean_str(&self) -> &str {
        let mut result = &self.inner;
        if result.starts_with('/') {
            result = &result[1..];
        }
        if result.ends_with('/') {
            result = &result[..1];
        }
        result
    }

    pub fn join<P: AsRef<ResourcePath>>(&self, other: P) -> ResourcePathBuf {
        let left = self.as_clean_str();
        let right = other.as_ref().as_clean_str();

        if left.is_empty() {
            return ResourcePathBuf::from(right.as_ref());
        }
        if right.is_empty() {
            return ResourcePathBuf::from(left.as_ref());
        }

        ResourcePathBuf {
            inner: [left, "/", right].concat(),
        }
    }

    pub fn to_filesystem_path(&self, root_dir: &::std::path::Path) -> ::std::path::PathBuf {
        let mut path: ::std::path::PathBuf = root_dir.into();

        for part in self.items() {
            path = path.join(sanitize_path_component(part).as_ref());
        }

        path
    }
}

impl ResourcePathBuf {
    pub fn from_filesystem_path(root_dir: &::std::path::Path, path: &::std::path::Path) -> Option<Self> {
        let relative_dir = path.strip_prefix(root_dir).ok()?;

        let mut path = ResourcePathBuf { inner: String::with_capacity(relative_dir.as_os_str().len() + 32) };
        for part in relative_dir.components() {
            path = path.join(unsanitize_path_component(part.as_os_str()).as_ref());
        }

        Some(path)
    }
}

// ---- Formatting ----

use std::fmt;

impl fmt::Debug for ResourcePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_tuple("ResourcePath").field(&&self.inner).finish()
    }
}

impl fmt::Display for ResourcePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl fmt::Debug for ResourcePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_tuple("ResourcePathBuf").field(&self.inner).finish()
    }
}

impl fmt::Display for ResourcePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&self.inner, f)
    }
}

// ---- Other utils ---

use std::borrow::Cow;

struct FixSolution<'s> {
    problematic_sequence_len: usize,
    fix: FixOutput<'s>,
}

enum FixOutput<'s> {
    /// Insert a slice to the byt output
    Single(&'s [u8]),
    /// Insert 3 slices to byte output
    Triple(&'s [u8], &'s [u8], &'s [u8]),
}

/// Check if the subsequent string requires a fix
///
/// The fixes here should be reversible. It should be possible to reconstruct the
/// resource name from the sanitized output.
fn check_for_sanitize_fix(previous_len: usize, remainder: &[u8]) -> Option<FixSolution> {
    let next_char = remainder[0];

    if previous_len == 0 && remainder.len() >= 3 {
        match next_char {
            b'C' | b'c' => {
                let c1 = remainder[1].to_ascii_lowercase();
                let c2 = remainder[2].to_ascii_lowercase();
                let c3 = remainder.iter().skip(3).next().cloned();
                let c4 = remainder.iter().skip(4).next().cloned();

                match (c1, c2, c3, c4) {
                    (b'o', b'n', None, None) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"_r", remainder, b"_"),
                        })
                    }
                    (b'o', b'n', Some(b'.'), _) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"_r", &remainder[..3], b"_"),
                        })
                    }
                    (b'o', b'm', Some(b'1'...b'9'), None) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 4,
                            fix: FixOutput::Triple(b"_r", remainder, b"_"),
                        })
                    }
                    (b'o', b'm', Some(b'1'...b'9'), Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 4,
                            fix: FixOutput::Triple(b"_r", &remainder[..4], b"_"),
                        })
                    }
                    _ => (),
                }
            }
            b'P' | b'p' => {
                let c1 = remainder[1].to_ascii_lowercase();
                let c2 = remainder[2].to_ascii_lowercase();
                let c3 = remainder.iter().skip(3).next().cloned();

                match (c1, c2, c3) {
                    (b'r', b'n', None) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"_r", remainder, b"_"),
                        })
                    }
                    (b'r', b'n', Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"_r", &remainder[..3], b"_"),
                        })
                    }
                    _ => (),
                }
            }
            b'A' | b'a' => {
                let c1 = remainder[1].to_ascii_lowercase();
                let c2 = remainder[2].to_ascii_lowercase();
                let c3 = remainder.iter().skip(3).next().cloned();

                match (c1, c2, c3) {
                    (b'u', b'x', None) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"_r", remainder, b"_"),
                        })
                    }
                    (b'u', b'x', Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"_r", &remainder[..3], b"_"),
                        })
                    }
                    _ => (),
                }
            }
            b'N' | b'n' => {
                let c1 = remainder[1].to_ascii_lowercase();
                let c2 = remainder[2].to_ascii_lowercase();
                let c3 = remainder.iter().skip(3).next().cloned();

                match (c1, c2, c3) {
                    (b'u', b'l', None) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"_r", remainder, b"_"),
                        })
                    }
                    (b'u', b'l', Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"_r", &remainder[..3], b"_"),
                        })
                    }
                    _ => (),
                }
            }
            b'L' | b'l' if remainder.len() >= 4 => {
                let c1 = remainder[1].to_ascii_lowercase();
                let c2 = remainder[2].to_ascii_lowercase();
                let c3 = remainder[3];
                let c4 = remainder.iter().skip(4).next().cloned();

                match (c1, c2, c3, c4) {
                    (b'p', b't', b'1'...b'9', None) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 4,
                            fix: FixOutput::Triple(b"_r", remainder, b"_"),
                        })
                    }
                    (b'p', b't', b'1'...b'9', Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 4,
                            fix: FixOutput::Triple(b"_r", &remainder[..4], b"_"),
                        })
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    match next_char {
        b'\\' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_b_"),
        }),
        b'_' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"__"),
        }),
        b'<' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_lt_"),
        }),
        b'>' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_gt_"),
        }),
        b':' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_c_"),
        }),
        b'\"' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_q_"),
        }),
        b'/' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_sl_"),
        }),
        b'|' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_p_"),
        }),
        b'?' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_m_"),
        }),
        b'*' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_a_"),
        }),
        i @ 1..=31 => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: match i {
                1 => FixOutput::Single(b"_i1_"),
                2 => FixOutput::Single(b"_i2_"),
                3 => FixOutput::Single(b"_i3_"),
                4 => FixOutput::Single(b"_i4_"),
                5 => FixOutput::Single(b"_i5_"),
                6 => FixOutput::Single(b"_i6_"),
                7 => FixOutput::Single(b"_i7_"),
                8 => FixOutput::Single(b"_i8_"),
                9 => FixOutput::Single(b"_i9_"),
                10 => FixOutput::Single(b"_i10_"),
                11 => FixOutput::Single(b"_i11_"),
                12 => FixOutput::Single(b"_i12_"),
                13 => FixOutput::Single(b"_i13_"),
                14 => FixOutput::Single(b"_i14_"),
                15 => FixOutput::Single(b"_i15_"),
                16 => FixOutput::Single(b"_i16_"),
                17 => FixOutput::Single(b"_i17_"),
                18 => FixOutput::Single(b"_i18_"),
                19 => FixOutput::Single(b"_i19_"),
                20 => FixOutput::Single(b"_i20_"),
                21 => FixOutput::Single(b"_i21_"),
                22 => FixOutput::Single(b"_i22_"),
                23 => FixOutput::Single(b"_i23_"),
                24 => FixOutput::Single(b"_i24_"),
                25 => FixOutput::Single(b"_i25_"),
                26 => FixOutput::Single(b"_i26_"),
                27 => FixOutput::Single(b"_i27_"),
                28 => FixOutput::Single(b"_i28_"),
                29 => FixOutput::Single(b"_i29_"),
                30 => FixOutput::Single(b"_i30_"),
                31 => FixOutput::Single(b"_i31_"),
                _ => unreachable!("should be in range 1 - 31"),
            },
        }),
        b'.' if remainder.len() == 1 => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_d_"),
        }),
        b' ' if remainder.len() == 1 => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"_s_"),
        }),
        _ => None,
    }
}

enum SanitizeState {
    /// Nothing was encountered that would need fixing
    Good { position: usize },
    /// Something was fixed, and the buffer for fixes was allocated
    Fixed { buffer: Vec<u8> },
}

/// Apply the fix based on previous sanitization state and fix output that was returned from requires_sanitize_fix
fn apply_sanitize_fix(
    problematic_sequence_len: usize,
    replacement: FixOutput,
    remainder: &mut &[u8],
    state: SanitizeState,
    all_bytes: &[u8],
) -> SanitizeState {
    match state {
        SanitizeState::Fixed { mut buffer } => {
            match replacement {
                FixOutput::Single(replacement) => buffer.extend_from_slice(replacement),
                FixOutput::Triple(a, b, c) => {
                    buffer.extend_from_slice(a);
                    buffer.extend_from_slice(b);
                    buffer.extend_from_slice(c);
                }
            }
            *remainder = &remainder[problematic_sequence_len..];
            SanitizeState::Fixed { buffer }
        }
        SanitizeState::Good { position } => {
            let mut buffer = Vec::with_capacity(1024);
            buffer.extend_from_slice(&all_bytes[..position]);
            match replacement {
                FixOutput::Single(replacement) => buffer.extend_from_slice(replacement),
                FixOutput::Triple(a, b, c) => {
                    buffer.extend_from_slice(a);
                    buffer.extend_from_slice(b);
                    buffer.extend_from_slice(c);
                }
            }
            *remainder = &remainder[problematic_sequence_len..];
            SanitizeState::Fixed { buffer }
        }
    }
}

/// Create a version of path string that is safe to use as filesystem path component,
/// provided that it is not empty.
pub fn sanitize_path_component(component: &str) -> Cow<str> {
    let bytes = component.as_bytes();
    let mut remainder = bytes;
    let mut state = SanitizeState::Good { position: 0 };

    'main: loop {
        state = match state {
            SanitizeState::Good { .. } => {
                let mut index = 0;
                loop {
                    if remainder.len() == 0 {
                        return Cow::from(component);
                    }

                    if let Some(s) = check_for_sanitize_fix(index, remainder) {
                        state = apply_sanitize_fix(
                            s.problematic_sequence_len,
                            s.fix,
                            &mut remainder,
                            SanitizeState::Good { position: index },
                            bytes,
                        );
                        continue 'main;
                    }

                    index += 1;
                    remainder = &remainder[1..];
                }
            }
            SanitizeState::Fixed { mut buffer } => {
                if remainder.len() == 0 {
                    return Cow::from(
                        String::from_utf8(buffer).expect("expected valid utf8 sequence"),
                    );
                }

                if let Some(s) = check_for_sanitize_fix(buffer.len(), remainder) {
                    apply_sanitize_fix(
                        s.problematic_sequence_len,
                        s.fix,
                        &mut remainder,
                        SanitizeState::Fixed { buffer },
                        bytes,
                    )
                } else {
                    buffer.extend_from_slice(&remainder[..1]);
                    remainder = &remainder[1..];
                    SanitizeState::Fixed { buffer }
                }
            }
        };
    }
}

use std::ffi::OsStr;

pub fn unsanitize_path_component(component: &OsStr) -> Cow<str> {
    #[derive(Copy, Clone)]
    enum FixState {
        Underscore,
        Scan,
    }

    enum UnsanitizeState {
        Fixed { bytes: Vec<u8>, state: FixState, position: usize },
        ReuseSameString,
    }

    let part = component.to_string_lossy();

    if part.len() == 0 {
        return part;
    }

    let state = {
        let bytes = part.as_ref().as_bytes();
        let bytes_len = bytes.len();

        let mut position = 0;

        loop {
            if bytes[position] == b'_' {
                let mut ok_data = Vec::with_capacity(bytes_len);
                ok_data.extend(bytes.iter().take(position));
                break UnsanitizeState::Fixed { bytes: ok_data, state: FixState::Underscore, position: position + 1 };
            }
            position += 1;
            if position >= bytes_len {
                break UnsanitizeState::ReuseSameString;
            }
        }
    };

    match state {
        UnsanitizeState::ReuseSameString => return part,
        UnsanitizeState::Fixed { mut bytes, mut state, mut position } => {
            let src_bytes = part.as_ref().as_bytes();
            let src_bytes_len = src_bytes.len();

            loop {
                match state {
                    FixState::Underscore => {
                        let remaining_len = src_bytes_len - position;

                        if remaining_len == 0 {
                            bytes.push(b'_'); // invalid file
                            break;
                        }

                        let next_char = src_bytes[position];

                        if remaining_len > 0 && next_char == b'_' {
                            bytes.push(b'_');
                            position += 1;
                            state = FixState::Scan;
                        } else if remaining_len > 4 && next_char == b'r' && src_bytes[position + 4] == b'_' {
                            bytes.extend_from_slice(&src_bytes[position + 1..position + 4]);
                            position += 5;
                            state = FixState::Scan;
                        } else if remaining_len > 5 && next_char == b'r' && src_bytes[position + 5] == b'_' {
                            bytes.extend_from_slice(&src_bytes[position + 1..position + 5]);
                            position += 6;
                            state = FixState::Scan;
                        } else if remaining_len > 2 && next_char == b'i' {
                            let next_char2 = src_bytes[position + 1];
                            let next_char3 = src_bytes[position + 2];

                            match (next_char2, next_char3) {
                                (b'1', b'_') => bytes.push(1),
                                (b'2', b'_') => bytes.push(2),
                                (b'3', b'_') => bytes.push(3),
                                (b'4', b'_') => bytes.push(4),
                                (b'5', b'_') => bytes.push(5),
                                (b'6', b'_') => bytes.push(6),
                                (b'7', b'_') => bytes.push(7),
                                (b'8', b'_') => bytes.push(8),
                                (b'9', b'_') => bytes.push(9),
                                _ => if remaining_len > 3 {
                                    let next_char4 = src_bytes[position + 3];

                                    match (next_char2, next_char3, next_char4) {
                                        (b'1', b'0', b'_') => bytes.push(10),
                                        (b'1', b'1', b'_') => bytes.push(11),
                                        (b'1', b'2', b'_') => bytes.push(12),
                                        (b'1', b'3', b'_') => bytes.push(13),
                                        (b'1', b'4', b'_') => bytes.push(14),
                                        (b'1', b'5', b'_') => bytes.push(15),
                                        (b'1', b'6', b'_') => bytes.push(16),
                                        (b'1', b'7', b'_') => bytes.push(17),
                                        (b'1', b'8', b'_') => bytes.push(18),
                                        (b'1', b'9', b'_') => bytes.push(19),
                                        (b'2', b'0', b'_') => bytes.push(20),
                                        (b'2', b'1', b'_') => bytes.push(21),
                                        (b'2', b'2', b'_') => bytes.push(22),
                                        (b'2', b'3', b'_') => bytes.push(23),
                                        (b'2', b'4', b'_') => bytes.push(24),
                                        (b'2', b'5', b'_') => bytes.push(25),
                                        (b'2', b'6', b'_') => bytes.push(26),
                                        (b'2', b'7', b'_') => bytes.push(27),
                                        (b'2', b'8', b'_') => bytes.push(28),
                                        (b'2', b'9', b'_') => bytes.push(29),
                                        (b'3', b'0', b'_') => bytes.push(30),
                                        (b'3', b'1', b'_') => bytes.push(31),
                                        _ => {
                                            bytes.push(b'_');
                                            bytes.extend_from_slice(&src_bytes[position..]); // invalid file
                                            break;
                                        },
                                    }

                                    position += 1;
                                },
                            }

                            position += 3;
                            state = FixState::Scan;
                        } else if remaining_len > 1 {
                            let next_char2 = src_bytes[position + 1];

                            match (next_char, next_char2) {
                                (b'd', b'_') => bytes.push(b'.'),
                                (b'b', b'_') => bytes.push(b'\\'),
                                (b'c', b'_') => bytes.push(b':'),
                                (b'q', b'_') => bytes.push(b'\"'),
                                (b'p', b'_') => bytes.push(b'|'),
                                (b'm', b'_') => bytes.push(b'?'),
                                (b'a', b'_') => bytes.push(b'*'),
                                (b's', b'_') => bytes.push(b' '),
                                _ => if remaining_len > 2 {
                                    let next_char3 = src_bytes[position + 2];

                                    match (next_char, next_char2, next_char3) {
                                        (b'l', b't', b'_') => bytes.push(b'<'),
                                        (b'g', b't', b'_') => bytes.push(b'>'),
                                        (b's', b'l', b'_') => bytes.push(b'/'),
                                        _ => {
                                            bytes.push(b'_');
                                            bytes.extend_from_slice(&src_bytes[position..]); // invalid file
                                        },
                                    }

                                    position += 1;
                                },
                            }

                            position += 2;
                            state = FixState::Scan;
                        } else {
                            bytes.push(b'_');
                            bytes.extend_from_slice(&src_bytes[position..]); // invalid file
                            break;
                        }
                    },
                    FixState::Scan => {
                        if position == src_bytes_len {
                            break;
                        }

                        let next_char = src_bytes[position];

                        if next_char == b'_' {
                            state = FixState::Underscore;
                        } else {
                            bytes.push(next_char);
                        }

                        position += 1;
                    }
                }
            }

            Cow::from(String::from_utf8(bytes).expect("bytes already undergone lossy conversion to utf8"))
        }
    }
}

#[cfg(test)]
mod normalize_path_tests {
    use super::{sanitize_path_component, unsanitize_path_component};
    use std::ffi::OsString;

    fn check(sanitized: &str, unsanitized: &str) {
        assert_eq!(sanitized, sanitize_path_component(unsanitized).as_ref());
        assert_eq!(unsanitized, unsanitize_path_component(&OsString::from(sanitized)).as_ref());
    }

    #[test]
    fn test_common() {
        // this is not valid path, but not a concern of this function
        check("", "");

        // _ is the start of the escape sequence, so this escapes the escape sequence
        check("__", "_");
        check("____", "__");

        // kill path traversing
        check("_d_", ".");
        check("._d_", "..");

        // simple unsanitized names
        check("hello world", "hello world");
        check("hello-world", "hello-world");
        check("hello+world", "hello+world");
    }

    #[test]
    fn test_windows() {
        check("_b_", "\\");
        check("_b__b_", "\\\\");

        check("_lt_", "<");
        check("_lt__lt_", "<<");

        check("_gt_", ">");
        check("_gt__gt_", ">>");

        check("_c_", ":");
        check("_c__c_", "::");

        check("_q_", "\"");
        check("_q__q_", "\"\"");

        check("_sl_", "/");
        check("_sl__sl_", "//");

        check("_p_", "|");
        check("_p__p_", "||");

        check("_m_", "?");
        check("_m__m_", "??");

        check("_a_", "*");
        check("_a__a_", "**");

        for i in 1u8..=31 {
            let mut output = String::new();
            output.push_str("_i");
            output.push_str(&format!("{}", i));
            output.push('_');

            let mut input = String::new();
            input.push(i as char);

            check(&output, &input);

            let mut output = String::new();
            output.push_str("_i");
            output.push_str(&format!("{}", i));
            output.push('_');
            output.push_str("_i");
            output.push_str(&format!("{}", i));
            output.push('_');

            let mut input = String::new();
            input.push(i as char);
            input.push(i as char);

            check(&output, &input);
        }

        check("hello_s_", "hello ");
        check("hello_d_", "hello.");
        check("hello _s_", "hello  ");
        check("hello._d_", "hello..");
        check(" hello _s_", " hello  ");
        check(".hello._d_", ".hello..");

        for reserved_name in &[
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ] {
            let seq = format!("{}", &reserved_name);
            check(
                &format!("_r{}_", seq),
                &seq
            );

            let seq = format!("{}", reserved_name.to_lowercase());
            check(
                &format!("_r{}_", seq),
                &seq
            );

            let seq = format!("{}", title_case(reserved_name));
            check(
                &format!("_r{}_", seq),
                &seq
            );

            let seq = format!("{}", &reserved_name);
            let input = format!("{}.txt", &seq);
            let output = format!("_r{}_.txt", &seq);
            check(&output, &input);

            let seq = format!("{}", &reserved_name);
            let input = format!("{}.", &seq);
            let output = format!("_r{}__d_", &seq);
            check(&output, &input);

            let seq = format!("{}", &reserved_name);
            let input = format!("{}.a", &seq);
            let output = format!("_r{}_.a", &seq);
            check(&output, &input);

            let seq = format!("{}", &reserved_name);
            let input = format!("hi {} and bye", &seq);
            let output = format!("hi {} and bye", &seq);
            check(&output, &input);
        }
    }

    fn title_case(value: &str) -> String {
        value
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i == 0 {
                    Box::new(c.to_uppercase()) as Box<Iterator<Item = char>>
                } else {
                    Box::new(c.to_lowercase()) as Box<Iterator<Item = char>>
                }
            })
            .collect()
    }
}
