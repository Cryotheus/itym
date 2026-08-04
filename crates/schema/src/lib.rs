/// `struct`
pub unsafe trait StructSchema {
	type Size;
}

/// `enum`
pub unsafe trait EnumSchema {
	type Size;

	const VARIANTS: u16;
}

/// `union`
pub unsafe trait UnionSchema {
	type Size;

	const VARIANTS: u16;
}

/// `(T0, ...)`
pub unsafe trait TupleSchema {
	const LEN: usize;
}

pub unsafe trait TupleSchemaField<const INDEX: usize>: TupleSchema {
	type Field;
}

pub unsafe trait TupleSchemaPop: TupleSchema {
	type Popped;
	type Residual;
}

pub unsafe trait TupleSchemaPush: TupleSchema {
	type Pushed<A>;
}

unsafe impl TupleSchema for () {
	const LEN: usize = 0;
}

unsafe impl TupleSchemaPush for () {
	type Pushed<A> = (A,);
}

// macro template
unsafe impl<T0> TupleSchema for (T0,) {
	const LEN: usize = 1;
}

unsafe impl<T0> TupleSchemaField<0> for (T0,) {
	type Field = T0;
}

unsafe impl<T0> TupleSchemaPop for (T0,) {
	type Popped = T0;
	type Residual = ();
}

unsafe impl<T0> TupleSchemaPush for (T0,) {
	type Pushed<A> = (T0, A);
}
