use cpython::_detail::ffi::PyTypeObject;
use cpython::{py_class, py_class_call_slot, py_fn, PyIterator, PyString};
use cpython::{NoArgs, ObjectProtocol, PyNone, ToPyObject};
use cpython::{
    PyDict, PyList, PyModule, PyObject, PyResult, Python, PythonObjectDowncastError,
    _detail::ffi,
    _detail::ffi::{vectorcallfunc, PyAsyncMethods, Py_ssize_t},
    _detail::from_owned_ptr_or_panic,
    pyobject_newtype,
};
use log::info;
use spvn_serde::ToPy;
use std::io::Bytes;

use cpython::PyClone;

pub struct Awaitable(PyAsyncMethods);

// impl Awaitable {}
// impl ToPyObject for Awaitable {
//     type ObjectType = PyObject;

//     fn to_py_object(&self, py: Python) -> PyObject {
//         unsafe {
//             let v = PyNone.into_py_object(py).as_ptr();

//             let ptr: *mut ffi::PyObject = ffi::_PyObject_New(v);
//             let t = from_owned_ptr_or_panic(py, ptr);

//             t
//         }
//     }
// }

pub struct Caller {
    pub app: PyObject,
}

pub trait Call<T: ToPy<PyDict>> {
    fn call(self, py: Python, scope: T);
}

impl<T: ToPy<PyDict>> Call<T> for Caller {
    fn call(self, py: Python, scope: T) {
        // py.allow_threads(|| {

        // });
        // let cast = to_py_object(py, scope);
        // let py_scope = match cast {
        //     Ok(pyobj) => pyobj,
        //     Err(e) => panic!("{:#?}", e),
        // };

        // let args = PyTuple::new(py, &[]);

        fn send(py: Python, a: PyDict) -> PyResult<PyNone> {
            #[cfg(debug_assertions)]
            info!("{:#?}", a.items(py));
            // let res = Awaitable(PyAsyncMethods {
            //     am_await: fn(*mut ffi::PyObject) -> *mut ffi::PyObject {

            //     },
            //     am_aiter: (),
            //     am_anext: (),
            // });
            // fn finish(py: Python) -> PyResult<bool> {
            //     Ok(true)
            // // }
            // let _bootstrap_ok = res
            //     ._unsafe_inner
            //     .set_item(py, "__await__", py_fn!(py, finish()));

            // #[cfg(debug_assertions)]
            // info!("bootstrapped: {:#?}", _bootstrap_ok);

            Ok(PyNone)
        }

        fn receive(_: Python) -> PyResult<Vec<u8>> {
            Ok(vec![1, 2, 3])
        }

        let kwargs = PyDict::new(py);

        kwargs.set_item(py, "scope", scope.to(py));
        kwargs.set_item(py, "send", py_fn!(py, send(a: PyDict)));
        kwargs.set_item(py, "receive", py_fn!(py, receive()));

        let result = self.app.call(py, NoArgs, Some(&kwargs));
        #[cfg(debug_assertions)]
        log::info!("call result {:#?}", result);

        let awa = match result {
            Ok(succ) => succ,
            Err(e) => panic!("{:#?}", e),
        };

        let hasawait = awa.getattr(py, "__await__");
        #[cfg(debug_assertions)]
        log::info!("await {:#?}", hasawait);

        let awaitable = match hasawait {
            Ok(succ) => succ,
            Err(e) => panic!("{:#?}", e),
        };

        let res = awaitable.call(py, NoArgs, None);
        #[cfg(debug_assertions)]
        log::info!("await result {:#?}", res);

        let awaitable = match res {
            Ok(succ) => succ,
            Err(e) => panic!("{:#?}", e),
        };

        #[cfg(debug_assertions)]
        log::info!("coroutine wrapper {:#?}", awaitable);

        let it = awaitable.iter(py);
        let mut await_result = match it {
            Ok(succ) => succ,
            Err(e) => panic!("{:#?}", e),
        };

        #[cfg(debug_assertions)]
        log::info!("result iterator");

        let mut n = 0;
        let mut py_result: Option<Result<PyObject, cpython::PyErr>> = None;
        loop {
            let sized = await_result.next();
            py_result = match sized {
                Some(obj) => Some(obj),
                None => break,
            };
            n += 1;

            match py_result.unwrap() {
                Ok(result) => result,
                Err(e) => {
                    info!("{:#?}", e);
                    break;
                }
            };

            #[cfg(debug_assertions)]
            log::info!("iters {:#?}", n);
        }
    }
}

impl From<PyObject> for Caller {
    fn from(app: PyObject) -> Self {
        Caller { app }
    }
}
