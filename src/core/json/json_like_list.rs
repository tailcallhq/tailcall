use super::JsonLike;

pub trait JsonLikeList<'json>: JsonLike<'json> {
    fn map<Err>(self, mut mapper: impl FnMut(Self) -> Result<Self, Err>) -> Result<Self, Err> {
        if self.as_array().is_some() {
            let new = self
                .into_array()
                .unwrap()
                .into_iter()
                .map(mapper)
                .collect::<Result<_, _>>()?;

            Ok(Self::array(new))
        } else {
            mapper(self)
        }
    }

    fn try_for_each<Err>(&self, mut f: impl FnMut(&Self) -> Result<(), Err>) -> Result<(), Err> {
        if let Some(arr) = self.as_array() {
            arr.iter().try_for_each(f)
        } else {
            f(self)
        }
    }
}

impl<'json, T: JsonLike<'json>> JsonLikeList<'json> for T {}
