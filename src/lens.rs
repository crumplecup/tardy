use std::sync::Arc;
use winit::window;

/// The `lens` module provides the [`Lens`] struct, which holds an application view and methods for
/// interacting with the view.
///
/// # Representing window state with `Lens`
///
/// The purpose of the `Lens` struct is to keep track of application state, the analogue of the
/// `WindowState` struct in the `windows` example of the [`winit`] crate.  Since the data in this
/// struct provides a view of the application to the user, the function is similar to that of
/// lenses in a pair of glasses.  The last time I tried this, I ended up with an `EguiState` and a
/// `GalileoState` and a `WindowState`, and it was starting to feel political, so I went for
/// whimsy.
///
/// This struct ends up as a catch-all holding data intended for display, interactivity flags, and
/// anything else that might come in handy. But for now, it just has a handle to the window, and an
/// optimistic `refresh` flag that isn't wired up to anything yet. As a beginner with
/// [egui]("https://docs.rs/egui/latest/egui/"), I
/// frequently insert these kind of control flags into a struct because the framework renders the
/// view anew every frame.  These flags indicate the need to perform an expensive operation, like
/// loading spatial data to a map, and should only happen once, so I will add a boolean field to
/// the struct to track this granular detail of the application space.
///
/// Eventually I want to be able to share a window between the well-tested `egui` library and the
/// relatively immature [galileo](https://docs.rs/galileo/latest/galileo/) library, but for now we
/// are just stubbing this out for future use by wrapping it in an [`Arc`].
#[derive(Debug, derive_getters::Getters, derive_setters::Setters)]
#[setters(prefix = "with_", into, borrow_self)]
pub struct Lens {
    refresh: bool,
    window: Arc<window::Window>,
}

impl Lens {
    /// The `new` method creates an instance of `Lens` from an [`Arc<window::Window>`].
    pub fn new(window: Arc<window::Window>) -> Self {
        Self {
            refresh: false,
            window,
        }
    }
}
