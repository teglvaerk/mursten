extern crate nalgebra;
extern crate rand;

use std::marker::PhantomData;


pub struct Game<Bk, Scn> 
where
    Bk: Backend<Scn>,
    Scn: Scene,
{
    backend: Bk,
    phantom: PhantomData<Scn>,
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
    Bk: Backend<Scn>,
    Scn: Scene,
{
    pub fn new(backend: Bk) -> Self {
        Self {
            backend, phantom: PhantomData
        }
    }

    pub fn run(self, scene: Scn) -> Scn {
        let Game {
            backend,
            ..
        } = self;

        backend.run(scene)
    }
}

pub trait Backend<Scn>
where
    Self: Sized,
    Scn: Scene,
{
    fn run(
        self,
        scene: Scn
    ) -> Scn;

    fn quit(&mut self);
}


pub struct NullBackend<Scn> {
    must_quit: bool,
    _data: Option<Scn>,
}

impl<Scn> NullBackend<Scn> {
    pub fn new() -> Self {
        Self { must_quit: false, _data: None }
    }
}

impl<Scn> Backend<Scn> for NullBackend<Scn>
where 
    Self: Sized,
    Scn: Scene + logic::Update<()> + graphics::Draw<()>,
{
    fn run(
        self,
        mut scene: Scn
    ) -> Scn {
        while !self.must_quit {
            scene.update(&mut ());
            if self.must_quit {
                return scene
            }
            scene.draw(&mut ());
        }
        scene
    }
    fn quit(&mut self) {
        self.must_quit = true;
    }
}

pub mod logic {
    pub trait Update<Ctx> {
        fn update(&mut self, context: &mut Ctx);
    }
    pub trait ElapsedDelta {
        fn delta(&self) -> f32;
    }
}

pub mod graphics {
    use nalgebra::*;

    pub trait Color: Clone + Copy {
        fn into_rgba(self) -> [f32; 4];
    }

    pub enum DrawMode {
        Line(f32),
        Fill,
    }

    pub trait Graphics {
        fn clear<C: Color>(&mut self, C);
        fn present(&mut self);
        // fn draw(Drawable, position: Point2, scale: f32);
    }

    pub trait DrawPrimitives: Graphics {
        fn set_color<C: Color>(&mut self, C);
        fn circle(&mut self, mode: DrawMode, origin: Point2<f32>, radius: f32);
        fn ellipse(&mut self, mode: DrawMode, origin: Point2<f32>, width: f32, height: f32);
        fn line(&mut self, origin: Point2<f32>, target: Point2<f32>, width: f32);
        fn polygon(&mut self, mode: DrawMode, points: &Vec<Point2<f32>>);
        fn square(&mut self, mode: DrawMode, up_left: Point2<f32>, width: f32) {
            self.rectangle(mode, up_left, width, width);
        }
        fn rectangle(&mut self, mode: DrawMode, up_left: Point2<f32>, width: f32, height: f32) {
            self.polygon(mode, &vec![
                up_left,
                up_left + Vector2::new(width, 0.0),
                up_left + Vector2::new(width, height),
                up_left + Vector2::new(0.0, height),
            ]);
        }
        fn square_centered(&mut self, mode: DrawMode, center: Point2<f32>, width: f32) {
            self.square(mode, center - Vector2::new(width/2.0, width/2.0), width);
        }
        fn text(&mut self, position: Point2<f32>, text: &str);
        // fn text_centered(&mut self, position: Vector2<f32>, text: &str);
    }
    
    pub struct PushTransform<'scr, Scr: 'scr> {
        s: &'scr mut Scr,
        transform: Transform2<f32>,
    }
    
    impl<'scr, Scr> PushTransform<'scr, Scr>
    where
        Scr: 'scr + DrawPrimitives,
    {
        pub fn new(s: &'scr mut Scr, transform: Transform2<f32>) -> Self {
            PushTransform { s, transform }
        }
    }

    impl<'scr, Scr> Graphics for PushTransform<'scr, Scr>
    where
        Scr: 'scr + Graphics,
    {
        fn clear<C: Color>(&mut self, color: C) {
            self.s.clear(color);
        }
        fn present(&mut self) {
            self.s.present();
        }
    }
    
    impl<'scr, Scr> DrawPrimitives for PushTransform<'scr, Scr>
    where
        Scr: 'scr + DrawPrimitives,
    {
        fn set_color<C: Color>(&mut self, color: C) {
            self.s.set_color(color);
        }
        fn circle(&mut self, mode: DrawMode, origin: Point2<f32>, radius: f32) {
            self.s.circle(mode, self.transform * origin, radius);
        }
        fn ellipse(&mut self, mode: DrawMode, origin: Point2<f32>, width: f32, height: f32) {
            self.s.ellipse(mode, self.transform * origin, width, height);
        }
        fn line(&mut self, origin: Point2<f32>, target: Point2<f32>, width: f32) {
            self.s.line(self.transform * origin, self.transform * target, width);
        }
        fn polygon(&mut self, mode: DrawMode, points: &Vec<Point2<f32>>) {
            let transform = self.transform;
            let points : Vec<_> = points.iter().map(|p| { transform * p }).collect();
            self.s.polygon(mode, &points);
        }
        fn text(&mut self, position: Point2<f32>, text: &str) {
            self.s.text(self.transform * position, text);
        }
    }

    pub trait Draw<Scr> {
        fn draw(&self, screen: &mut Scr);
    }
}

pub mod sequence {
    
    #[derive(Clone)]
    pub struct Sequence {
        current: u32,
    }

    impl Sequence {
        pub fn new() -> Sequence {
            Sequence {
                current: 0,
            }
        }
        pub fn step<'s, 'c, S, C>(&mut self, state: &'s mut S, context: &'c mut C) -> SecuenceExecuter<'s, 'c, S, C> {
            self.current += 1;
            SecuenceExecuter {
                state,
                context,
                execute_at_step: 0,
                before: self.current - 1,
                now: self.current,
            }
        }
    }

    pub struct SecuenceExecuter<'s, 'c, S: 's, C: 'c> {
        before: u32,
        now: u32,
        execute_at_step: u32,
        state: &'s mut S,
        context: &'c mut C,
    }

    impl<'s, 'c, S, C> SecuenceExecuter<'s, 'c, S, C> {
        pub fn then<F>(self, c: F) -> Self
            where
                F: FnOnce(&mut S, &mut C),
        {
            if self.before <= self.execute_at_step && self.execute_at_step < self.now {
                c(self.state, self.context);
            }
            SecuenceExecuter {
                ..self
            }
        }
        pub fn wait(self, steps: u32) -> Self {
            SecuenceExecuter {
                execute_at_step: self.execute_at_step + steps,
                ..self
            }
        }
    }
    
    // use logic::Update;

    // impl<C> Update<C> for Sequence {
    //     fn update(&mut self, ctx: &mut C) {
    //     }
    // }
}

pub mod input {
    use nalgebra::*;
    
    pub type JoystickId = u32;
    
    pub trait JoystickProvider {
        fn joystick(&self, id: JoystickId) -> Joystick;
        fn available_joysticks(&self) -> Vec<JoystickId>;
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Button {
        Normal,
        JustPressed,
        BeingHeld,
        JustReleased,
    }
    
    impl Button {
        pub fn is_pressed(&self) -> bool {
            match self {
                Button::JustPressed | Button::BeingHeld => true,
                _ => false
            }
        }
        pub fn is_not_pressed(&self) -> bool {
            !self.is_pressed()
        }
    }
    
    impl From<bool> for Button {
        fn from(state: bool) -> Self {
            if state { Button::BeingHeld } else { Button::Normal }
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Dpad {
        Up,
        Right,
        Bottom,
        Left,
    }

    impl<'a> Into<Vector2<f32>> for &'a Dpad {
        fn into(self) -> Vector2<f32> {
            match self {
                Dpad::Up     => Vector2::new( 0.0,  1.0),
                Dpad::Right  => Vector2::new( 1.0,  0.0),
                Dpad::Bottom => Vector2::new( 0.0, -1.0),
                Dpad::Left   => Vector2::new(-1.0,  0.0),
            }
        }
    }

    impl Into<Vector2<f32>> for Dpad {
        fn into(self) -> Vector2<f32> {
            match self {
                Dpad::Up     => Vector2::new( 0.0, -1.0),
                Dpad::Right  => Vector2::new( 1.0,  0.0),
                Dpad::Bottom => Vector2::new( 0.0,  1.0),
                Dpad::Left   => Vector2::new(-1.0,  0.0),
            }
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct Joystick {
        pub left_axis: Vector2<f32>,
        pub left_axis_button: Button,
        pub right_axis: Vector2<f32>,
        pub right_axis_button: Button,
        pub d_pad: Option<Dpad>,
        pub a: Button,
        pub b: Button,
        pub x: Button,
        pub y: Button,
        pub left_bumper: Button,
        pub left_trigger: Button,
        pub left_trigger_pressure: f32,
        pub right_bumper: Button,
        pub right_trigger: Button,
        pub right_trigger_pressure: f32,
        pub start: Button,
        pub back: Button,
    }
    
    impl Default for Joystick {
        fn default() -> Self {
            Joystick {
                left_axis: Vector2::zeros(),
                left_axis_button: Button::Normal,
                right_axis: Vector2::zeros(),
                right_axis_button: Button::Normal,
                d_pad: None,
                a: Button::Normal,
                b: Button::Normal,
                x: Button::Normal,
                y: Button::Normal,
                left_bumper: Button::Normal,
                left_trigger: Button::Normal,
                left_trigger_pressure: 0.0,
                right_bumper: Button::Normal,
                right_trigger: Button::Normal,
                right_trigger_pressure: 0.0,
                start: Button::Normal,
                back: Button::Normal,
            }
        }
    }
}


pub mod random {
    use rand::{SeedableRng, RngCore, thread_rng};
    use rand::rngs::SmallRng;
    use rand::distributions::{Distribution, Normal, Uniform, Poisson};

    #[derive(Clone, PartialEq, Eq)]
    pub struct Seed(u64);

    impl Seed {
        pub fn new(seed: u64) -> Self {
            Seed(seed)
        }
        
        pub fn rng(&self) -> Rng {
            Rng(SmallRng::seed_from_u64(self.0))
        }
        
        pub fn random() -> Seed {
            Seed(thread_rng().next_u64())
        }
    }
    
    pub struct Rng(SmallRng);
    
    impl Rng {
        pub fn random() -> Rng {
            Seed::random().rng()
        }
        
        pub fn poisson(&mut self, rate: f32) -> f32{
            Poisson::new(rate.into()).sample(&mut self.0) as f32
        }
        
        pub fn normal(&mut self, mean: f32, stdev: f32) -> f32 {
            Normal::new(mean.into(), stdev.into()).sample(&mut self.0) as f32
        }
        pub fn triangular(&mut self, low: f32, high: f32, mode: f32) -> f32 {
            let u = Uniform::new(0.0, 1.0).sample(&mut self.0) as f32;
            let domain = high - low;
            let f = (mode - low) / domain;
            if u < f {
                low + (u * domain * (mode - low)).sqrt()
            }
            else {
                high - ((1.0 - u) * domain * (high - mode)).sqrt()
            }
        }
        pub fn seed(&mut self) -> Seed {
            Seed(self.0.next_u64())
        }
    }
}