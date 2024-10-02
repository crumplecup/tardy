pub trait Identifier<Value> {
    type Value;
    fn next(&mut self) -> Value;
}

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Deref,
    derive_more::DerefMut,
)]
pub struct Counter(u64);

impl Identifier<u64> for Counter {
    type Value = u64;
    fn next(&mut self) -> Self::Value {
        self.0 += 1;
        self.0
    }
}

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Deref,
    derive_more::DerefMut,
)]
pub struct Id<T: Identifier<U>, U> {
    #[deref]
    #[deref_mut]
    id: T,
    _phantom: std::marker::PhantomData<U>,
}

impl Id<Counter, u64> {
    pub fn counter() -> Self {
        let id = Counter::default();
        let _phantom = std::marker::PhantomData;
        Self { id, _phantom }
    }

    pub fn node_id(&mut self) -> accesskit::NodeId {
        let id = self.next();
        accesskit::NodeId(id)
    }
}
