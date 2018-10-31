pub use render::{RenderChain, Renderer};
pub use update::{UpdateChain, Updater};

pub struct Application<B, D> {
    backend: B,
    update_chain: UpdateChain<B, D>,
    render_chain: RenderChain<B, D>,
}

pub trait Data
where
    Self: Sized,
{
    // TODO: Unimplement this and make scenes implement this explicitly, then try to make a custom
    // derive for this trait.
    fn alive(&self) -> bool {
        true
    }
}

pub trait Backend<D>
where
    Self: Sized,
    D: Data,
{
    fn run(
        self,
        UpdateChain<Self, D>,
        RenderChain<Self, D>,
        D
    ) -> D;
    fn quit(&mut self);
}

impl<B, D> Application<B, D>
where
    B: Backend<D>,
    D: Data,
{
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            update_chain: UpdateChain::default(),
            render_chain: RenderChain::default(),
        }
    }

    pub fn run(self, data: D) -> D {
        let Application {
            backend,
            update_chain,
            render_chain,
        } = self;

        backend.run(update_chain, render_chain, data)
    }

    pub fn add_updater<U: 'static + Updater<B, D>>(mut self, updater: U) -> Self {
        self.update_chain.add(updater);
        self
    }

    pub fn add_renderer<R: 'static + Renderer<B, D>>(mut self, renderer: R) -> Self {
        self.render_chain.add(renderer);
        self
    }
}

pub mod update {
    use Backend;
    use Data;

    pub trait Updater<B, D>
    where
        D: Data,
    {
        fn update(&mut self, backend: &mut B, data: &mut D);
    }

    pub struct UpdateChain<B, D> {
        updaters: Vec<Box<Updater<B, D>>>,
    }

    impl<B, D> Default for UpdateChain<B, D> {
        fn default() -> Self {
            Self {
                updaters: Vec::new(),
            }
        }
    }

    impl<B, D> UpdateChain<B, D>
    where
        B: Backend<D>,
        D: Data,
    {
        pub fn add<U: 'static + Updater<B, D>>(&mut self, updater: U) {
            self.updaters.push(Box::new(updater));
        }
        pub fn update(&mut self, mut backend: &mut B, data: &mut D) {
            for u in self.updaters.iter_mut() {
                u.update(&mut backend, data);
            }
        }
    }
}

mod render {
    use Backend;
    use Data;

    pub trait Renderer<B, D>
    where
        D: Data,
    {
        fn render(&mut self, backend: &mut B, data: &D);
    }

    pub struct RenderChain<B, D> {
        renderers: Vec<Box<Renderer<B, D>>>,
    }

    impl<B, D> Default for RenderChain<B, D> {
        fn default() -> Self {
            Self {
                renderers: Vec::new(),
            }
        }
    }

    impl<B, D> RenderChain<B, D>
    where
        B: Backend<D>,
        D: Data,
    {
        pub fn add<R: 'static + Renderer<B, D>>(&mut self, renderer: R) {
            self.renderers.push(Box::new(renderer));
        }
        pub fn render(&mut self, mut backend: &mut B, data: &D) {
            for r in self.renderers.iter_mut() {
                r.render(&mut backend, data);
            }
        }
    }
}

pub mod dummy {
    pub struct DummyBackend<D> {
        must_quit: bool,
        _data: Option<D>,
    }

    impl<D> DummyBackend<D> {
        pub fn new() -> Self {
            Self { must_quit: false, _data: None }
        }
    }

    impl<D> super::Backend<D> for DummyBackend<D>
    where 
        Self: Sized,
        D: super::Data,
    {
        fn run(
            mut self,
            mut uc: super::UpdateChain<Self, D>,
            mut rc: super::RenderChain<Self, D>,
            mut data: D
        ) -> D {
            while !self.must_quit {
                uc.update(&mut self, &mut data);
                if self.must_quit {
                    return data
                }
                rc.render(&mut self, &data);
            }
            data
        }
        fn quit(&mut self) {
            self.must_quit = true;
        }
    }
}
