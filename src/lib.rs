pub struct Game<Bk, Scn> 
where
    Bk: backend::Backend<Scn>,
    Scn: Scene,
{
    backend: Bk,
    update_chain: backend::UpdateChain<Bk::Context, Scn>,
    render_chain: backend::RenderChain<Bk::Screen, Scn>,
}

pub trait Scene
where
    Self: Sized,
{
    fn alive(&self) -> bool {
        true
    }
}

impl<Bk, Scn> Game<Bk, Scn>
where
    Bk: backend::Backend<Scn>,
    Scn: Scene,
{
    pub fn new(backend: Bk) -> Self {
        Self {
            backend,
            update_chain: backend::UpdateChain::default(),
            render_chain: backend::RenderChain::default(),
        }
    }

    pub fn run(self, scene: Scn) -> Scn {
        let Game {
            backend,
            update_chain,
            render_chain,
        } = self;

        backend.run(update_chain, render_chain, scene)
    }

    pub fn add_updater<U: 'static + Updater<Bk::Context, Scn>>(mut self, updater: U) -> Self {
        self.update_chain.add(updater);
        self
    }

    pub fn add_renderer<R: 'static + Renderer<Bk::Screen, Scn>>(mut self, renderer: R) -> Self {
        self.render_chain.add(renderer);
        self
    }
}

pub trait Updater<Ctx, Scn>
where
    Scn: Scene,
{
    fn update(&mut self, context: &mut Ctx, scene: &mut Scn);
}

pub trait Renderer<Scr, Scn>
where
    Scn: Scene,
{
    fn render(&mut self, screen: &mut Scr, scene: &Scn);
}

pub mod backend {
    use Scene;
    use Updater;
    use Renderer;

    pub trait Backend<Scn>
    where
        Self: Sized,
        Scn: Scene,
    {
        type Context;
        type Screen;

        fn run(
            self,
            UpdateChain<Self::Context, Scn>,
            RenderChain<Self::Screen, Scn>,
            Scn
        ) -> Scn;

        fn quit(&mut self);
    }

    pub struct UpdateChain<Ctx, Scn> {
        updaters: Vec<Box<Updater<Ctx, Scn>>>,
    }

    impl<Ctx, Scn> Default for UpdateChain<Ctx, Scn> {
        fn default() -> Self {
            Self {
                updaters: Vec::new(),
            }
        }
    }

    impl<Ctx, Scn> UpdateChain<Ctx, Scn>
    where
        Scn: Scene,
    {
        pub fn add<U: 'static + Updater<Ctx, Scn>>(&mut self, updater: U) {
            self.updaters.push(Box::new(updater));
        }
        pub fn update(&mut self, context: &mut Ctx, scene: &mut Scn) {
            for u in self.updaters.iter_mut() {
                u.update(context, scene);
            }
        }
    }

    pub struct RenderChain<Scr, Scn> {
        renderers: Vec<Box<Renderer<Scr, Scn>>>,
    }

    impl<Scr, Scn> Default for RenderChain<Scr, Scn> {
        fn default() -> Self {
            Self {
                renderers: Vec::new(),
            }
        }
    }

    impl<Scr, Scn> RenderChain<Scr, Scn>
    where
        Scn: Scene,
    {
        pub fn add<R: 'static + Renderer<Scr, Scn>>(&mut self, renderer: R) {
            self.renderers.push(Box::new(renderer));
        }
        pub fn render(&mut self, screen: &mut Scr, scene: &Scn) {
            for r in self.renderers.iter_mut() {
                r.render(screen, scene);
            }
        }
    }
}

pub mod dummy_backend {
    use backend::{Backend, UpdateChain, RenderChain};

    pub struct DummyBackend<Scn> {
        must_quit: bool,
        _data: Option<Scn>,
    }

    impl<Scn> DummyBackend<Scn> {
        pub fn new() -> Self {
            Self { must_quit: false, _data: None }
        }
    }

    impl<Scn> Backend<Scn> for DummyBackend<Scn>
    where 
        Self: Sized,
        Scn: super::Scene,
    {
        type Context = ();
        type Screen = ();

        fn run(
            self,
            mut uc: UpdateChain<(), Scn>,
            mut rc: RenderChain<(), Scn>,
            mut data: Scn
        ) -> Scn {
            while !self.must_quit {
                uc.update(&mut (), &mut data);
                if self.must_quit {
                    return data
                }
                rc.render(&mut (), &data);
            }
            data
        }
        fn quit(&mut self) {
            self.must_quit = true;
        }
    }
}
