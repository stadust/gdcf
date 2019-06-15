use diesel::insertable::Insertable;

#[derive(Debug)]
pub(crate) struct Wrapped<T>(pub(crate) T);

impl<'a, T, Tab> Insertable<Tab> for Wrapped<&'a T>
where
    &'a T: Insertable<Tab>,
{
    type Values = <&'a T as Insertable<Tab>>::Values;

    fn values(self) -> Self::Values {
        self.0.values()
    }
}
