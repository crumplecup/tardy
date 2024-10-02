use crate::Lens;

#[derive(derive_more::From)]
pub enum Event {
    #[from(accesskit_winit::Event)]
    Access(accesskit_winit::Event),
    #[from(Lens)]
    Lens(Lens),
}
