use super::pytype::PyTypeRef;
use crate::function::Args;
use crate::iterator;
use crate::pyobject::{PyClassImpl, PyContext, PyObjectRef, PyRef, PyResult, PyValue};
use crate::vm::VirtualMachine;

/// map(func, *iterables) --> map object
///
/// Make an iterator that computes the function using arguments from
/// each of the iterables.  Stops when the shortest iterable is exhausted.
#[pyclass(module = false, name = "map")]
#[derive(Debug)]
pub struct PyMap {
    mapper: PyObjectRef,
    iterators: Vec<PyObjectRef>,
}

impl PyValue for PyMap {
    fn class(vm: &VirtualMachine) -> PyTypeRef {
        vm.ctx.types.map_type.clone()
    }
}

#[pyimpl(flags(BASETYPE))]
impl PyMap {
    #[pyslot]
    fn tp_new(
        cls: PyTypeRef,
        function: PyObjectRef,
        iterables: Args,
        vm: &VirtualMachine,
    ) -> PyResult<PyRef<Self>> {
        let iterators = iterables
            .into_iter()
            .map(|iterable| iterator::get_iter(vm, &iterable))
            .collect::<Result<Vec<_>, _>>()?;
        PyMap {
            mapper: function,
            iterators,
        }
        .into_ref_with_type(vm, cls)
    }

    #[pymethod(name = "__next__")]
    fn next(&self, vm: &VirtualMachine) -> PyResult {
        let next_objs = self
            .iterators
            .iter()
            .map(|iterator| iterator::call_next(vm, iterator))
            .collect::<Result<Vec<_>, _>>()?;

        // the mapper itself can raise StopIteration which does stop the map iteration
        vm.invoke(&self.mapper, next_objs)
    }

    #[pymethod(name = "__iter__")]
    fn iter(zelf: PyRef<Self>) -> PyRef<Self> {
        zelf
    }

    #[pymethod(name = "__length_hint__")]
    fn length_hint(&self, vm: &VirtualMachine) -> PyResult<usize> {
        self.iterators.iter().try_fold(0, |prev, cur| {
            let cur = iterator::length_hint(vm, cur.clone())?.unwrap_or(0);
            let max = std::cmp::max(prev, cur);
            Ok(max)
        })
    }
}

pub fn init(context: &PyContext) {
    PyMap::extend_class(context, &context.types.map_type);
}
