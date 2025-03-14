/// a wrapper for `Vec<>` taken from <https://github.com/Cysharp/csbindgen>
#[repr(C)]
pub struct ByteBuffer {
    pub(crate) ptr: *mut u8,
    length: i32,
}

impl ByteBuffer {
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn len(&self) -> usize {
        self.length
            .try_into()
            .expect("buffer length negative or overflowed")
    }

    pub fn from_vec(bytes: Vec<u8>) -> Self {
        let length = i32::try_from(bytes.len()).expect("buffer length cannot fit into a i32.");

        // keep memory until call delete
        let mut v = std::mem::ManuallyDrop::new(bytes);

        Self {
            ptr: v.as_mut_ptr(),
            length,
        }
    }

    pub fn from_vec_struct<T: Sized>(bytes: Vec<T>) -> Self {
        let element_size = std::mem::size_of::<T>() as i32;
        let length = (bytes.len() as i32) * element_size;
        let mut v = std::mem::ManuallyDrop::new(bytes);

        Self {
            ptr: v.as_mut_ptr() as *mut u8,
            length,
        }
    }

    pub fn destroy_into_vec(self) -> Vec<u8> {
        if self.ptr.is_null() {
            vec![]
        } else {
            let length: usize = self
                .length
                .try_into()
                .expect("buffer length negative or overflowed");

            unsafe { Vec::from_raw_parts(self.ptr, length, length) }
        }
    }

    pub fn destroy_into_vec_struct<T: Sized>(self) -> Vec<T> {
        if self.ptr.is_null() {
            vec![]
        } else {
            let element_size = std::mem::size_of::<T>() as i32;
            let length = (self.length * element_size) as usize;

            unsafe { Vec::from_raw_parts(self.ptr as *mut T, length, length) }
        }
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.ptr, self.len())
    }

    pub fn destroy(self) {
        drop(self.destroy_into_vec());
    }
}

/// # Safety
/// Expects that buffer is a valid pointer to a ByteBuffer
#[no_mangle]
pub unsafe extern "C" fn free_byte_buffer(buffer: *mut ByteBuffer) {
    let buf = Box::from_raw(buffer);
    // drop inner buffer, if you need Vec<u8>, use buf.destroy_into_vec() instead.
    buf.destroy();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let buffer = ByteBuffer::from_vec(data);

        assert_eq!(buffer.len(), 5);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_from_struct_vec() {
        #[allow(dead_code)]
        struct Foo {
            a: i32,
            b: String,
        }

        let data = vec![
            Foo {
                a: 1,
                b: "hello".to_string(),
            },
            Foo {
                a: 2,
                b: "world".to_string(),
            },
        ];
        let buffer = ByteBuffer::from_vec_struct(data);
        assert_eq!(buffer.len(), 64);
    }
}
