#[derive(Debug)]
pub struct Cache<A> {
    stored: Option<A>,
}

impl<A> Cache<A> {
    pub fn new() -> Self {
        Self { stored: None }
    }

    pub fn take(&mut self) -> Option<A> {
        self.stored.take()
    }

    pub fn take_or<F>(&mut self, create: F) -> A
    where
        F: FnOnce() -> A,
    {
        self.take().unwrap_or_else(create)
    }

    pub fn store(&mut self, val: A) {
        self.stored = Some(val);
    }
}

impl<A> Default for Cache<A> {
    fn default() -> Self {
        Self::new()
    }
}
