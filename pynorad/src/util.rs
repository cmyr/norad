use norad::{Color, Identifier};
use std::str::FromStr;

use pyo3::{
    exceptions::{PyIndexError, PyValueError},
    PyResult,
};

#[macro_export]
macro_rules! flatten {
    ($expr:expr $(,)?) => {
        match $expr {
            Err(e) => Err(e),
            Ok(Err(e)) => Err(e),
            Ok(Ok(fine)) => Ok(fine),
        }
    };
}

/// A helper macro that creates a proxy object referencing a vec field of
/// another proxy object.
#[macro_export]
macro_rules! seq_proxy {
    ($name:ident, $inner:ty, $member:ty, $field:ident, $concrete:ty) => {
        #[pyclass]
        #[derive(Debug, Clone)]
        pub struct $name {
            pub(crate) inner: $inner,
        }

        impl $name {
            pub(crate) fn with<R>(
                &self,
                f: impl FnOnce(&Vec<$concrete>) -> R,
            ) -> Result<R, $crate::ProxyError> {
                self.inner.with(|x| f(&x.$field))
            }

            pub(crate) fn with_mut<R>(
                &mut self,
                f: impl FnOnce(&mut Vec<$concrete>) -> R,
            ) -> Result<R, $crate::ProxyError> {
                self.inner.with_mut(|x| f(&mut x.$field))
            }
        }

        #[pyproto]
        impl pyo3::PySequenceProtocol for $name {
            fn __len__(&self) -> usize {
                self.inner.with(|x| x.$field.len()).unwrap_or(0)
            }

            fn __getitem__(&'p self, idx: isize) -> pyo3::PyResult<$member> {
                let idx = $crate::util::python_idx_to_idx(idx, self.__len__())?;
                self.with(|x| <$member>::new(self.clone(), idx, x[idx].py_id)).map_err(Into::into)
            }

            fn __delitem__(&'p mut self, idx: isize) -> pyo3::PyResult<()> {
                let idx = $crate::util::python_idx_to_idx(idx, self.__len__())?;
                self.with_mut(|x| x.remove(idx))?;
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! seq_proxy_member {
    ($name:ident, $parent:ident, $concrete:ty, $err:ident) => {
        #[pyclass]
        #[derive(Debug, Clone)]
        pub struct $name {
            pub(crate) inner: $parent,
            pub(crate) idx: std::cell::Cell<usize>,
            py_id: norad::PyId,
        }

        impl $name {
            fn new(inner: $parent, idx: usize, py_id: norad::PyId) -> Self {
                $name { inner, idx: Cell::new(idx), py_id }
            }

            fn with<R>(&self, f: impl FnOnce(&$concrete) -> R) -> Result<R, $crate::ProxyError> {
                $crate::flatten!(self.inner.with(|x| match x.get(self.idx.get()) {
                    Some(pt) if pt.py_id == self.py_id => Some(pt),
                    _ => match x.iter().enumerate().find(|(_, pt)| pt.py_id == self.py_id) {
                        Some((i, pt)) => {
                            self.idx.set(i);
                            Some(pt)
                        }
                        None => None,
                    },
                }
                .ok_or_else(|| $crate::ProxyError::$err(self.clone()))
                .map(|g| f(g))))
            }

            fn with_mut<R>(
                &mut self,
                f: impl FnOnce(&mut $concrete) -> R,
            ) -> Result<R, $crate::ProxyError> {
                let $name { inner, py_id, idx } = self;
                let result = inner.with_mut(|x| match x.get_mut(idx.get()) {
                    Some(pt) if pt.py_id == *py_id => Some(f(pt)),
                    _ => match x.iter_mut().enumerate().find(|(_, pt)| pt.py_id == *py_id) {
                        Some((i, pt)) => {
                            idx.set(i);
                            Some(f(pt))
                        }
                        None => None,
                    },
                })?;

                match result {
                    Some(thing) => Ok(thing),
                    None => Err($crate::ProxyError::$err(self.clone())),
                }
            }
        }
    };
}

/// A helper macro for generating python iterators.
///
/// Works on types generated by `seq_proxy`.
#[macro_export]
macro_rules! seq_proxy_iter {
    ($name:ident, $parent:ident, $member:ty) => {
        #[pyproto]
        impl pyo3::PyIterProtocol for $parent {
            fn __iter__(slf: pyo3::PyRef<Self>) -> $name {
                $name { inner: slf.clone(), ix: 0 }
            }
        }

        #[pyclass]
        pub struct $name {
            inner: $parent,
            ix: usize,
        }

        #[pyproto]
        impl pyo3::PyIterProtocol for $name {
            fn __iter__(slf: pyo3::PyRef<'p, Self>) -> pyo3::PyRef<'p, Self> {
                slf
            }

            fn __next__(mut slf: pyo3::PyRefMut<Self>) -> Option<$member> {
                let index = slf.ix;
                slf.ix += 1;
                slf.inner.__getitem__(index as isize).ok()
            }
        }
    };
}

/// A helper macro that implements the python == and != ops
#[macro_export]
macro_rules! proxy_eq {
    ($name:ident) => {
        #[pyproto]
        impl pyo3::PyObjectProtocol for $name {
            fn __richcmp__(
                &'p self,
                other: PyRef<$name>,
                op: pyo3::class::basic::CompareOp,
            ) -> pyo3::PyResult<bool> {
                let other: &$name = &*other;
                match op {
                    pyo3::class::basic::CompareOp::Eq => {
                        flatten!(self.with(|x| other.with(|y| x == y))).map_err(Into::into)
                    }
                    pyo3::class::basic::CompareOp::Ne => {
                        flatten!(self.with(|x| other.with(|y| x != y))).map_err(Into::into)
                    }
                    _ => Err(pyo3::exceptions::PyNotImplementedError::new_err("")),
                }
            }
        }
    };
}

pub(crate) fn python_idx_to_idx(idx: isize, len: usize) -> PyResult<usize> {
    let idx = if idx.is_negative() { len - (idx.abs() as usize % len) } else { idx as usize };

    if idx < len {
        Ok(idx)
    } else {
        Err(PyIndexError::new_err(format!(
            "Index {} out of bounds of collection with length {}",
            idx, len
        )))
    }
}

pub(crate) fn to_identifier(s: Option<&str>) -> PyResult<Option<Identifier>> {
    s.map(Identifier::new).transpose().map_err(|_| {
        PyValueError::new_err(
            "Identifier must be between 0 and 100 characters, each in the range 0x20..=0x7E",
        )
    })
}

pub(crate) fn to_color(s: Option<&str>) -> PyResult<Option<Color>> {
    s.map(Color::from_str).transpose().map_err(|_| PyValueError::new_err("Invalid color string"))
}
