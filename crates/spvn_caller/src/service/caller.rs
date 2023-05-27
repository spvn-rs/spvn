use cpython::{PyObject, PyTuple, Python};
use spvn_cfg::ASGIScope;

#[allow(improper_ctypes)]
extern "C" {
    fn PyObject_CallObject(callable: &PyObject, args: &PyTuple) -> *mut PyObject;
}

pub struct Caller {
    pub app: PyObject,
}


pub trait Target {
    fn with_target(self, py: Python, tgt: &str);
}
impl Target for Caller {
    fn with_target(self, py: Python, tgt: &str) {
        
    }
}

pub trait Call {
    fn call(self, py: Python);
}

impl Call for Caller {
    fn call(self, py: Python) {
        // let cast = to_py_object(py, scope);
        // let py_scope = match cast {
        //     Ok(pyobj) => pyobj,
        //     Err(e) => panic!("{:#?}", e),
        // };

        let args = PyTuple::new(py, &[]);

        let result: *mut PyObject;
        unsafe {
            result = PyObject_CallObject(&self.app, &args);
        }
        #[cfg(debug_assertions)]
        log::info!("{:#?}", result)
    }
}

impl From<PyObject> for Caller {
    fn from(value: PyObject) -> Self {
        Caller { app: value }
    }
}
