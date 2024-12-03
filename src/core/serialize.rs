pub trait Serialize {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> where Self: Sized;
}

impl Serialize for Vec<u8> {
    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }

    #[inline]
    fn from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> where Self: Sized {
        Some(bytes.as_ref().to_vec())
    }
}

impl Serialize for String {
    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    #[inline]
    fn from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> where Self: Sized {
        String::from_utf8(bytes.as_ref().to_vec()).ok()
    }
}
