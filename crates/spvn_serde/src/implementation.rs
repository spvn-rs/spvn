use pyo3::{prelude::Python, types::PyDict, ToPyObject, IntoPy, Py, PyAny};
use serde::{Deserialize, Serialize};

static SPEC_VERSION: &str = "2.3";
static ASGI_VERSION: &str = "2.3";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ASGIVersions {
    spec_version: &'static str,
    version: &'static str,
}

impl IntoPy<Py<PyAny>> for ASGIVersions { 
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        let dict = PyDict::new(py);
        set_dict_item_feedback(py, &dict, "spec_version", self.spec_version);
        set_dict_item_feedback(py, &dict, "version", self.version);
        dict.to_object(py)
    }
}

pub fn set_dict_item_feedback<K: ToPyObject, V: ToPyObject>(
    _py: Python,
    dict: &PyDict,
    k: K,
    v: V,
) {
    let _res = dict.set_item(k, v);
}

pub const ASGI_IMPLEMENTATION: fn() -> ASGIVersions = || -> ASGIVersions {
    ASGIVersions {
        spec_version: SPEC_VERSION,
        version: ASGI_VERSION,
    }
};
