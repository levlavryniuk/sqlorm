/// Marker trait for types (primitives, tuples) intended for the generic Executor.
///
/// This trait acts as a positive compile-time check. The generic `Executor`
/// implementation is constrained to types that implement `GenericRow`.
///
/// Application-specific models like `User` or `Post` which have their own
/// custom executors should *not* implement this trait. This prevents the
/// compiler from finding a conflicting implementation and ensures the more
/// specific executor is chosen for those types.
pub trait GenericRow {}

impl GenericRow for String {}
impl GenericRow for &str {}
impl GenericRow for i8 {}
impl GenericRow for i16 {}
impl GenericRow for i32 {}
impl GenericRow for i64 {}
impl GenericRow for u8 {}
impl GenericRow for u16 {}
impl GenericRow for u32 {}
impl GenericRow for u64 {}
impl GenericRow for f32 {}
impl GenericRow for f64 {}
impl GenericRow for bool {}
impl<T> GenericRow for Option<T> where T: GenericRow + Send {}

// If you use other common crates, you can add implementations here under a feature flag.
// #[cfg(feature = "chrono")]
// impl GenericRow for chrono::NaiveDateTime {}
#[cfg(feature = "uuid")]
impl GenericRow for uuid::Uuid {}

/// A macro to implement `GenericRow` for tuples of varying arity.
macro_rules! impl_generic_row_for_tuples {
    ($($T:ident),+) => {
        impl<$($T: Send),+> GenericRow for ($($T,)+) {}
    };
}

impl_generic_row_for_tuples!(T1);
impl_generic_row_for_tuples!(T1, T2);
impl_generic_row_for_tuples!(T1, T2, T3);
impl_generic_row_for_tuples!(T1, T2, T3, T4);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5, T6);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5, T6, T7);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_generic_row_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_generic_row_for_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
impl_generic_row_for_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
);
