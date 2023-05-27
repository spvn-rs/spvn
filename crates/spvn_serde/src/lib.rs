use cpython::Python;

pub trait ToPy<T> {
    fn to(self, py: Python) -> T;
}
