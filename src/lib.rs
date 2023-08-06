use std::convert::TryFrom;

pub mod ir_gen;
pub mod llvm;
pub mod parser;
pub mod lexer;

pub const SMALL_STR_SIZE: usize = 16;

#[derive(Debug, PartialEq)]
pub struct SmalLCStr([u8; SMALL_STR_SIZE]);

impl SmalLCStr {
    pub fn new<T: AsRef<[u8]>>(src: &T) -> Option<SmalLCStr> {
        let src = src.as_ref();
        let len = src.len();

        let contains_null = unsafe { !libc::memchr(src.as_ptr() as *const libc::c_void, 0, len).is_null() };
        if contains_null || len >= SMALL_STR_SIZE {
            None
        } else {
            let mut buf = [0; SMALL_STR_SIZE];
            buf[..len].copy_from_slice(src);
            Some(SmalLCStr(buf))
        }
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
}

impl TryFrom<&str> for SmalLCStr {
    type Error = ();

    fn try_from(src: &str) -> Result<SmalLCStr, ()> {
        SmalLCStr::new(src).ok_or(())
    }
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}