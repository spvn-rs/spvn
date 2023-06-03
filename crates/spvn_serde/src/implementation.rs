use pyo3::{prelude::Python, types::PyDict, ToPyObject};
use serde::{Deserialize, Serialize};

static SpecVersion: &str = "2.3";
static AsgiVersion: &str = "2.3";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ASGIVersions {
    spec_version: String,
    version: String,
}

impl ASGIVersions {
    pub fn to_object(self, py: Python<'_>) -> pyo3::PyObject {
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
        spec_version: String::from(SpecVersion),
        version: String::from(AsgiVersion),
    }
};
