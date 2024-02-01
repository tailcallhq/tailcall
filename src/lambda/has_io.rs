pub trait HasIO {
    fn has_io(&self) -> bool;
}

impl<T> HasIO for Box<T>
where
    T: HasIO,
{
    fn has_io(&self) -> bool {
        self.as_ref().has_io()
    }
}

impl<T> HasIO for &Box<T>
where
    T: HasIO,
{
    fn has_io(&self) -> bool {
        self.as_ref().has_io()
    }
}

impl<T> HasIO for Vec<T>
where
    T: HasIO,
{
    fn has_io(&self) -> bool {
        self.iter().any(|elem| elem.has_io())
    }
}

impl<T> HasIO for &Vec<T>
where
    T: HasIO,
{
    fn has_io(&self) -> bool {
        self.iter().any(|elem| elem.has_io())
    }
}

impl<T1, T2> HasIO for (T1, T2)
where
    T1: HasIO,
    T2: HasIO,
{
    fn has_io(&self) -> bool {
        self.0.has_io() || self.1.has_io()
    }
}

impl<T1, T2, T3> HasIO for (T1, T2, T3)
where
    T1: HasIO,
    T2: HasIO,
    T3: HasIO,
{
    fn has_io(&self) -> bool {
        self.0.has_io() || self.1.has_io() || self.2.has_io()
    }
}
