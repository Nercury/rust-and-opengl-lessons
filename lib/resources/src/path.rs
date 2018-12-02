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
            if let Some(unsanitized_part) = unsanitize_path_component(part.as_os_str()) {
                path = path.join(unsanitized_part.as_ref());
            } else {
                return None;
            }
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
                            fix: FixOutput::Triple(b"+r", remainder, b"+"),
                        })
                    }
                    (b'o', b'n', Some(b'.'), _) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"+r", &remainder[..3], b"+"),
                        })
                    }
                    (b'o', b'm', Some(b'1'...b'9'), None) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 4,
                            fix: FixOutput::Triple(b"+r", remainder, b"+"),
                        })
                    }
                    (b'o', b'm', Some(b'1'...b'9'), Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 4,
                            fix: FixOutput::Triple(b"+r", &remainder[..4], b"+"),
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
                            fix: FixOutput::Triple(b"+r", remainder, b"+"),
                        })
                    }
                    (b'r', b'n', Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"+r", &remainder[..3], b"+"),
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
                            fix: FixOutput::Triple(b"+r", remainder, b"+"),
                        })
                    }
                    (b'u', b'x', Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"+r", &remainder[..3], b"+"),
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
                            fix: FixOutput::Triple(b"+r", remainder, b"+"),
                        })
                    }
                    (b'u', b'l', Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 3,
                            fix: FixOutput::Triple(b"+r", &remainder[..3], b"+"),
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
                            fix: FixOutput::Triple(b"+r", remainder, b"+"),
                        })
                    }
                    (b'p', b't', b'1'...b'9', Some(b'.')) => {
                        return Some(FixSolution {
                            problematic_sequence_len: 4,
                            fix: FixOutput::Triple(b"+r", &remainder[..4], b"+"),
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
            fix: FixOutput::Single(b"+b+"),
        }),
        b'+' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"++"),
        }),
        b'<' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+lt+"),
        }),
        b'>' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+gt+"),
        }),
        b':' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+c+"),
        }),
        b'\"' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+q+"),
        }),
        b'/' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+sl+"),
        }),
        b'|' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+p+"),
        }),
        b'?' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+m+"),
        }),
        b'*' => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+a+"),
        }),
        i @ 1..=31 => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: match i {
                1 => FixOutput::Single(b"+i1+"),
                2 => FixOutput::Single(b"+i2+"),
                3 => FixOutput::Single(b"+i3+"),
                4 => FixOutput::Single(b"+i4+"),
                5 => FixOutput::Single(b"+i5+"),
                6 => FixOutput::Single(b"+i6+"),
                7 => FixOutput::Single(b"+i7+"),
                8 => FixOutput::Single(b"+i8+"),
                9 => FixOutput::Single(b"+i9+"),
                10 => FixOutput::Single(b"+i10+"),
                11 => FixOutput::Single(b"+i11+"),
                12 => FixOutput::Single(b"+i12+"),
                13 => FixOutput::Single(b"+i13+"),
                14 => FixOutput::Single(b"+i14+"),
                15 => FixOutput::Single(b"+i15+"),
                16 => FixOutput::Single(b"+i16+"),
                17 => FixOutput::Single(b"+i17+"),
                18 => FixOutput::Single(b"+i18+"),
                19 => FixOutput::Single(b"+i19+"),
                20 => FixOutput::Single(b"+i20+"),
                21 => FixOutput::Single(b"+i21+"),
                22 => FixOutput::Single(b"+i22+"),
                23 => FixOutput::Single(b"+i23+"),
                24 => FixOutput::Single(b"+i24+"),
                25 => FixOutput::Single(b"+i25+"),
                26 => FixOutput::Single(b"+i26+"),
                27 => FixOutput::Single(b"+i27+"),
                28 => FixOutput::Single(b"+i28+"),
                29 => FixOutput::Single(b"+i29+"),
                30 => FixOutput::Single(b"+i30+"),
                31 => FixOutput::Single(b"+i31+"),
                _ => unreachable!("should be in range 1 - 31"),
            },
        }),
        b'.' if remainder.len() == 1 => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+d+"),
        }),
        b' ' if remainder.len() == 1 => Some(FixSolution {
            problematic_sequence_len: 1,
            fix: FixOutput::Single(b"+s+"),
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

pub fn unsanitize_path_component(component: &OsStr) -> Option<Cow<str>> {
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
        return Some(part);
    }

    let state = {
        let bytes = part.as_ref().as_bytes();
        let bytes_len = bytes.len();

        let mut position = 0;

        loop {
            if bytes[position] == b'+' {
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
        UnsanitizeState::ReuseSameString => return Some(part),
        UnsanitizeState::Fixed { mut bytes, mut state, mut position } => {
            let src_bytes = part.as_ref().as_bytes();
            let src_bytes_len = src_bytes.len();

            loop {
                match state {
                    FixState::Underscore => {
                        let remaining_len = src_bytes_len - position;

                        if remaining_len == 0 {
                            return None;
                        }

                        let next_char = src_bytes[position];

                        if remaining_len > 0 && next_char == b'+' {
                            bytes.push(b'+');
                            position += 1;
                            state = FixState::Scan;
                        } else if remaining_len > 4 && next_char == b'r' && src_bytes[position + 4] == b'+' {
                            bytes.extend_from_slice(&src_bytes[position + 1..position + 4]);
                            position += 5;
                            state = FixState::Scan;
                        } else if remaining_len > 5 && next_char == b'r' && src_bytes[position + 5] == b'+' {
                            bytes.extend_from_slice(&src_bytes[position + 1..position + 5]);
                            position += 6;
                            state = FixState::Scan;
                        } else if remaining_len > 2 && next_char == b'i' {
                            let next_char2 = src_bytes[position + 1];
                            let next_char3 = src_bytes[position + 2];

                            match (next_char2, next_char3) {
                                (b'1', b'+') => bytes.push(1),
                                (b'2', b'+') => bytes.push(2),
                                (b'3', b'+') => bytes.push(3),
                                (b'4', b'+') => bytes.push(4),
                                (b'5', b'+') => bytes.push(5),
                                (b'6', b'+') => bytes.push(6),
                                (b'7', b'+') => bytes.push(7),
                                (b'8', b'+') => bytes.push(8),
                                (b'9', b'+') => bytes.push(9),
                                _ => if remaining_len > 3 {
                                    let next_char4 = src_bytes[position + 3];

                                    match (next_char2, next_char3, next_char4) {
                                        (b'1', b'0', b'+') => bytes.push(10),
                                        (b'1', b'1', b'+') => bytes.push(11),
                                        (b'1', b'2', b'+') => bytes.push(12),
                                        (b'1', b'3', b'+') => bytes.push(13),
                                        (b'1', b'4', b'+') => bytes.push(14),
                                        (b'1', b'5', b'+') => bytes.push(15),
                                        (b'1', b'6', b'+') => bytes.push(16),
                                        (b'1', b'7', b'+') => bytes.push(17),
                                        (b'1', b'8', b'+') => bytes.push(18),
                                        (b'1', b'9', b'+') => bytes.push(19),
                                        (b'2', b'0', b'+') => bytes.push(20),
                                        (b'2', b'1', b'+') => bytes.push(21),
                                        (b'2', b'2', b'+') => bytes.push(22),
                                        (b'2', b'3', b'+') => bytes.push(23),
                                        (b'2', b'4', b'+') => bytes.push(24),
                                        (b'2', b'5', b'+') => bytes.push(25),
                                        (b'2', b'6', b'+') => bytes.push(26),
                                        (b'2', b'7', b'+') => bytes.push(27),
                                        (b'2', b'8', b'+') => bytes.push(28),
                                        (b'2', b'9', b'+') => bytes.push(29),
                                        (b'3', b'0', b'+') => bytes.push(30),
                                        (b'3', b'1', b'+') => bytes.push(31),
                                        _ => return None,
                                    }

                                    position += 1;
                                },
                            }

                            position += 3;
                            state = FixState::Scan;
                        } else if remaining_len > 1 {
                            let next_char2 = src_bytes[position + 1];

                            match (next_char, next_char2) {
                                (b'd', b'+') => bytes.push(b'.'),
                                (b'b', b'+') => bytes.push(b'\\'),
                                (b'c', b'+') => bytes.push(b':'),
                                (b'q', b'+') => bytes.push(b'\"'),
                                (b'p', b'+') => bytes.push(b'|'),
                                (b'm', b'+') => bytes.push(b'?'),
                                (b'a', b'+') => bytes.push(b'*'),
                                (b's', b'+') => bytes.push(b' '),
                                _ => if remaining_len > 2 {
                                    let next_char3 = src_bytes[position + 2];

                                    match (next_char, next_char2, next_char3) {
                                        (b'l', b't', b'+') => bytes.push(b'<'),
                                        (b'g', b't', b'+') => bytes.push(b'>'),
                                        (b's', b'l', b'+') => bytes.push(b'/'),
                                        _ => return None,
                                    }

                                    position += 1;
                                },
                            }

                            position += 2;
                            state = FixState::Scan;
                        } else { return None }
                    },
                    FixState::Scan => {
                        if position == src_bytes_len {
                            break;
                        }

                        let next_char = src_bytes[position];

                        if next_char == b'+' {
                            state = FixState::Underscore;
                        } else {
                            bytes.push(next_char);
                        }

                        position += 1;
                    }
                }
            }

            Some(Cow::from(String::from_utf8(bytes).expect("bytes already undergone lossy conversion to utf8")))
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

        // + is the start of the escape sequence, so this escapes the escape sequence
        check("++", "+");
        check("++++", "++");

        // kill path traversing
        check("+d+", ".");
        check(".+d+", "..");

        // simple unsanitized names
        check("hello world", "hello world");
        check("hello-world", "hello-world");
        check("hello_world", "hello_world");

        // underscore handling
        assert_eq!("quad+.vert", unsanitize_path_component(&OsString::from("quad+.vert")).as_ref());
    }

    #[test]
    fn test_windows() {
        check("+b+", "\\");
        check("+b++b+", "\\\\");

        check("+lt+", "<");
        check("+lt++lt+", "<<");

        check("+gt+", ">");
        check("+gt++gt+", ">>");

        check("+c+", ":");
        check("+c++c+", "::");

        check("+q+", "\"");
        check("+q++q+", "\"\"");

        check("+sl+", "/");
        check("+sl++sl+", "//");

        check("+p+", "|");
        check("+p++p+", "||");

        check("+m+", "?");
        check("+m++m+", "??");

        check("+a+", "*");
        check("+a++a+", "**");

        for i in 1u8..=31 {
            let mut output = String::new();
            output.push_str("+i");
            output.push_str(&format!("{}", i));
            output.push('+');

            let mut input = String::new();
            input.push(i as char);

            check(&output, &input);

            let mut output = String::new();
            output.push_str("+i");
            output.push_str(&format!("{}", i));
            output.push('+');
            output.push_str("+i");
            output.push_str(&format!("{}", i));
            output.push('+');

            let mut input = String::new();
            input.push(i as char);
            input.push(i as char);

            check(&output, &input);
        }

        check("hello+s+", "hello ");
        check("hello+d+", "hello.");
        check("hello +s+", "hello  ");
        check("hello.+d+", "hello..");
        check(" hello +s+", " hello  ");
        check(".hello.+d+", ".hello..");

        for reserved_name in &[
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ] {
            let seq = format!("{}", &reserved_name);
            check(
                &format!("+r{}+", seq),
                &seq
            );

            let seq = format!("{}", reserved_name.to_lowercase());
            check(
                &format!("+r{}+", seq),
                &seq
            );

            let seq = format!("{}", title_case(reserved_name));
            check(
                &format!("+r{}+", seq),
                &seq
            );

            let seq = format!("{}", &reserved_name);
            let input = format!("{}.txt", &seq);
            let output = format!("+r{}+.txt", &seq);
            check(&output, &input);

            let seq = format!("{}", &reserved_name);
            let input = format!("{}.", &seq);
            let output = format!("+r{}++d+", &seq);
            check(&output, &input);

            let seq = format!("{}", &reserved_name);
            let input = format!("{}.a", &seq);
            let output = format!("+r{}+.a", &seq);
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
