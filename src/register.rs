use core::marker::PhantomData;
use core::ops::Add;

use typenum::consts::{B1, U0};
use typenum::{IsGreater, IsGreaterOrEqual, IsLessOrEqual, Unsigned};

use type_bounds::num::runtime::Bounded;

pub trait ReadOnlyRegister {
    /// `get_field` takes a field and sets the value of that
    /// field to its value in the register.
    fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        f: Field<M, O, U>,
    ) -> Option<Field<M, O, U>>
    where
        U: IsGreater<U0, Output = B1>;

    /// `read` returns the current state of the register as a `u32`.
    fn read(&self) -> u32;

    /// `extract` pulls the state of a register out into a wrapped
    /// read-only register.
    fn extract(&self) -> ReadOnlyCopy;

    /// `is_set` takes a field and returns true if that field's value
    /// is equal to its upper bound or not. This is for particular use
    /// in single-bit fields.
    fn is_set<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        f: Field<M, O, U>,
    ) -> bool
    where
        U: IsGreater<U0, Output = B1>;

    /// `matches_any` returns whether or not any of the given fields
    /// match those fields values inside the register.
    fn matches_any<V: Positioned>(&self, val: V) -> bool;

    /// `matches_all` returns whether or not all of the given fields
    /// match those fields values inside the register.
    fn matches_all<V: Positioned>(&self, val: V) -> bool;
}

pub trait WriteOnlyRegister {
    /// `modify` takes one or more fields, joined by `+`, and
    /// sets those fields in the register, leaving the others
    /// as they were.
    fn modify<V: Positioned>(&mut self, val: V);

    /// `write` sets the value of the whole register to the
    /// given `u32` value.
    fn write(&mut self, val: u32);
}

pub trait ReadWriteRegister {
    /// `get_field` takes a field and sets the value of that
    /// field to its value in the register.
    fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        f: Field<M, O, U>,
    ) -> Option<Field<M, O, U>>
    where
        U: IsGreater<U0, Output = B1>;

    /// `read` returns the current state of the register as a `u32`.
    fn read(&self) -> u32;

    /// `extract` pulls the state of a register out into a wrapped
    /// read-only register.
    fn extract(&self) -> ReadOnlyCopy;

    /// `is_set` takes a field and returns true if that field's value
    /// is equal to its upper bound or not. This is for particular use
    /// in single-bit fields.
    fn is_set<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        f: Field<M, O, U>,
    ) -> bool
    where
        U: IsGreater<U0, Output = B1>;

    /// `matches_any` returns whether or not any of the given fields
    /// match those fields values inside the register.
    fn matches_any<V: Positioned>(&self, val: V) -> bool;

    /// `matches_all` returns whether or not all of the given fields
    /// match those fields values inside the register.
    fn matches_all<V: Positioned>(&self, val: V) -> bool;

    /// `modify` takes one or more fields, joined by `+`, and
    /// sets those fields in the register, leaving the others
    /// as they were.
    fn modify<V: Positioned>(&mut self, val: V);

    /// `write` sets the value of the whole register to the
    /// given `u32` value.
    fn write(&mut self, val: u32);
}

pub struct ReadOnlyCopy(pub u32);

impl ReadOnlyRegister for ReadOnlyCopy {
    /// `get_field` takes a field and sets the value of that
    /// field to its value in the register.
    fn get_field<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        f: Field<M, O, U>,
    ) -> Option<Field<M, O, U>>
    where
        U: IsGreater<U0, Output = B1>,
    {
        f.set((self.0 & M::U32) >> O::U32)
    }

    /// `read` returns the current state of the register as a `u32`.
    fn read(&self) -> u32 {
        self.0
    }

    /// `extract` pulls the state of a register out into a wrapped
    /// read-only register.
    fn extract(&self) -> ReadOnlyCopy {
        ReadOnlyCopy(self.0)
    }

    /// `is_set` takes a field and returns true if that field's value
    /// is equal to its upper bound or not. This is of particular use
    /// in single-bit fields.
    fn is_set<M: Unsigned, O: Unsigned, U: Unsigned>(
        &self,
        _: Field<M, O, U>,
    ) -> bool
    where
        U: IsGreater<U0, Output = B1>,
    {
        ((self.0 & M::U32) >> O::U32) == U::U32
    }

    /// `matches_any` returns whether or not any of the given fields
    /// match those fields values inside the register.
    fn matches_any<V: Positioned>(&self, val: V) -> bool {
        (val.in_position() & self.0) != 0
    }

    /// `matches_all` returns whether or not all of the given fields
    /// match those fields values inside the register.
    fn matches_all<V: Positioned>(&self, val: V) -> bool {
        (val.in_position() & self.0) == val.in_position()
    }
}

/// A field in a register parameterized by its mask, offset, and upper
/// bound. To construct a field, its `val` must be ⩽ `U::U32`.
///
/// It uses these type-level numbers so that the mask and offset can
/// be constant.
#[derive(Debug)]
pub struct Field<M: Unsigned, O: Unsigned, U: Unsigned>
where
    U: IsGreater<U0, Output = B1>,
{
    val: Bounded<u32, U0, U>,
    _mask: PhantomData<M>,
    _offset: PhantomData<O>,
}

impl<M: Unsigned, O: Unsigned, U: Unsigned> Field<M, O, U>
where
    U: IsGreater<U0, Output = B1>,
{
    /// New returns a `Some(Field)` if the given value is less than or equal to
    /// its upper bound, otherwise it returns `None`.
    pub fn new(val: u32) -> Option<Self> {
        Bounded::new(val).map(|val| Self {
            val: val,
            _offset: PhantomData,
            _mask: PhantomData,
        })
    }

    /// `set` takes an existing field sets its value to `val`. If val
    /// is _not_ ⩽ `U`, it returns `None`.
    pub fn set(mut self, val: u32) -> Option<Self> {
        Bounded::new(val).map(|val| {
            self.val = val;
            self
        })
    }

    /// `checked` is a compile-time checked constructor for a
    /// `Field`. Its `V` parameter must be ⩽ `U`; if it is not, the
    /// program will fail to typecheck.
    pub const fn checked<V: Unsigned>() -> Self
    where
        V: IsLessOrEqual<U, Output = B1>,
        V: IsGreaterOrEqual<U0, Output = B1>,
    {
        Self {
            val: Bounded::checked::<V>(),
            _offset: PhantomData,
            _mask: PhantomData,
        }
    }

    /// `val` retrieves the value from the field.
    pub fn val(&self) -> u32 {
        self.val.val
    }

    /// `is_set` returns whether or not the field's val is equal to
    /// its upper bound.
    pub fn is_set(&self) -> bool {
        self.val.val == U::U32
    }
}

impl<M: Unsigned, O: Unsigned, U: Unsigned> PartialEq<Field<M, O, U>>
    for Field<M, O, U>
where
    U: IsGreater<U0, Output = B1>,
{
    fn eq(&self, rhs: &Field<M, O, U>) -> bool {
        self.val() == rhs.val()
    }
}

/// `Positioned` is a trait which is used to represent a value, be it
/// a `Field` or simply a `u32`, as its value were it to be _in
/// position_ in its register.
///
/// It comes into play in `Register::modify` where, in the case of a
/// use like `Field1 + Field2 + Field3`, it is simply a no-op; as the
/// `+` implementation already positions the field. On the other hand,
/// when simply passing one `Field`, `in_position` will shift the
/// `Field`'s value right by `O`.
pub trait Positioned {
    fn mask(&self) -> u32;
    fn in_position(&self) -> u32;
}

impl<M: Unsigned, O: Unsigned, U: Unsigned> Positioned for Field<M, O, U>
where
    U: IsGreater<U0, Output = B1>,
{
    /// The mask for this positioned value.
    fn mask(&self) -> u32 {
        M::U32
    }

    /// Presents a value as its register-relative value.
    fn in_position(&self) -> u32 {
        self.val() << O::U32
    }
}

/// `FieldDisj` is short for _Field Disjunction_. It is a type which
/// constitutes the intermediate result of the summing, or disjunct of
/// two fields. It is not a type which one should use directly.
pub struct FieldDisj {
    mask: u32,
    val: u32,
}

impl Positioned for FieldDisj {
    fn mask(&self) -> u32 {
        self.mask
    }

    fn in_position(&self) -> u32 {
        self.val
    }
}

// Add where both lhs and rhs are `Field`s.
impl<
        LM: Unsigned,
        LO: Unsigned,
        LU: Unsigned,
        RM: Unsigned,
        RO: Unsigned,
        RU: Unsigned,
    > Add<Field<RM, RO, RU>> for Field<LM, LO, LU>
where
    LU: IsGreater<U0, Output = B1>,
    RU: IsGreater<U0, Output = B1>,
{
    type Output = FieldDisj;

    fn add(self, rhs: Field<RM, RO, RU>) -> Self::Output {
        FieldDisj {
            val: (self.val() << LO::U32) | (rhs.val() << RO::U32),
            mask: LM::U32 | RM::U32,
        }
    }
}

// Add where the rhs is a `FieldDisj`. This is necessary because I do
// not know which direction the compiler will associate `+`.
impl<M: Unsigned, O: Unsigned, U: Unsigned> Add<FieldDisj> for Field<M, O, U>
where
    U: IsGreater<U0, Output = B1>,
{
    type Output = FieldDisj;

    fn add(self, rhs: FieldDisj) -> Self::Output {
        FieldDisj {
            val: (self.val() << O::U32) | rhs.val,
            mask: M::U32 | rhs.mask(),
        }
    }
}

// Add where the lhs is a `FieldDisj`. This is necessary because I do
// not know which direction the compiler will associate `+`.
impl<M: Unsigned, O: Unsigned, U: Unsigned> Add<Field<M, O, U>> for FieldDisj
where
    U: IsGreater<U0, Output = B1>,
{
    type Output = FieldDisj;

    fn add(self, rhs: Field<M, O, U>) -> Self::Output {
        FieldDisj {
            val: self.val | (rhs.val() << O::U32),
            mask: self.mask | M::U32,
        }
    }
}

pub trait Pointer {
    unsafe fn ptr(&self) -> *mut u32;
}
