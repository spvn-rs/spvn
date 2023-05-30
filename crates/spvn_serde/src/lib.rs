
se pyo3::Python;

pub trait ToPy<'a, T> {
    fn to(self, py: Python<'a>) -> T;
}

// #[derive(Clone, Copy)]
// pub struct WeakRefDict<'a> {
//     ptr: std::ptr::NonNull<&'a PyDict>,
//     _stablize: std::marker::PhantomPinned,
// }

// impl From<&'_ PyDict> for WeakRefDict<'_> {
//     fn from(obj: &'_ PyDict) -> Self {
//         let mut memptr: *mut PyDict = std::ptr::null_mut();
//         unsafe {
//             let ret = libc::posix_memalign(
//                 (&mut memptr as *mut *mut PyDict).cast(),
//                 max(align_of::<&PyDict>(), size_of::<usize>()),
//                 size_of::<&PyDict>(),
//             );
//             assert_eq!(ret, 0, "Failed to allocate or invalid alignment");
//         };
//         let ptr = { ptr::NonNull::new(memptr).expect("posix_memalign should have returned 0") };
//         unsafe {
//             ptr.as_ptr().write(obj);
//         }
//         Self {
//             ptr,
//             _stablize: std::marker::PhantomPinned::default(),
//         }
//     }
// }
