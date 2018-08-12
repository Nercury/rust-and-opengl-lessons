#[derive(Clone)]
pub struct ResourcePathBuf {
    inner: String,
}

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
        ResourcePathBuf { inner: other.inner.into() }
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
            }
        }
    }

    pub fn to_string(&self) -> String {
        self.inner.into()
    }

    pub fn items(&self) -> impl Iterator<Item=&str> {
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

        if left.is_empty() { return ResourcePathBuf::from(right.as_ref()); }
        if right.is_empty() { return ResourcePathBuf::from(left.as_ref()); }

        ResourcePathBuf {
            inner: [left, "/", right].concat()
        }
    }
}

// ---- Formatting ----

use std::fmt;

impl fmt::Debug for ResourcePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_tuple("ResourcePath")
            .field(&&self.inner)
            .finish()
    }
}

impl fmt::Display for ResourcePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl fmt::Debug for ResourcePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_tuple("ResourcePathBuf")
            .field(&self.inner)
            .finish()
    }
}

impl fmt::Display for ResourcePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&self.inner, f)
    }
}

