use crate::{Act, Cmd, Lens};
use std::collections::HashMap;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::{
    event::{self, WindowEvent},
    event_loop, window,
};

/// The `app` module contains the `App` struct, which holds the parent-level top view of the
/// application state.
///
/// # Creating Windows with Tardy
///
/// The purpose of the `App` struct is to render content to the user in one or more windows.  There
/// is no content yet, because the [`winit`] library we are using to make windows does not do
/// anything else.  It is "a low-level brick in a hierarchy of libraries".
///
/// This code draws inspiration from the [window]("https://github.com/rust-windowing/winit/blob/master/examples/window.rs")
/// example in the [`winit`] repository (primary using \<CR-C\> and \<CR-V\>).  The main feature from
/// this example that I want to lift is the ability to create multiple windows.  Admittedly, I do
/// not have anything to put in these windows yet, but previously when I tackled this library, I
/// had trouble handling more than one window.  Today's goal: create as many windows as I want.
///
/// On April 27th of this year, the `winit` crate released version `0.30.0`, and I was eager to embrace the change.  
/// The examples I had pillaged my code from were all on a pre-0.30 version.  How hard could it be?
/// Turns out it boiled down to two key bits of *news* in the release notes:
///
/// * Deprecated EventLoop::run, use EventLoop::run_app.
/// * The new app APIs accept a newly added `ApplicationHandler<T>` instead of `Fn`
///
/// I.e. *Your old code will break*.  Some refactoring required.  This is a green-field space to
/// begin that refactoring process.
///
/// The second main goal of this refactor is make the application async from the ground up.  Too
/// many geospatial operations are compute heavy, and too many professional collaborations are over
/// network to run blocking operations.  Threads would be an acceptable alternative, but I am
/// interesting in biting into the shiny apple of async code, so here we go.
///
/// We decorate the main function with `#[tokio::main]`, using [`tokio`] for our runtime.  We will
/// be working with the `tokio` analogues to the `block_on`, `spawn_task`, `channel`, `join!` and
/// other building blocks from the `trpl` repository.  To begin with, we do not even call `await`,
/// so the runtime is just needless overhead. Let's go ahead and commit and I will start fixing
/// that.
#[derive(Debug, Default)]
pub struct App {
    cmd: Cmd,
    config: config::Config,
    windows: HashMap<window::WindowId, Lens>,
}

/// ### Fields
///
/// * The `cmd` field holds the [`Cmd`] struct, which maps keyboard inputs to program responses.
/// * The `config` field holds the [`config::Config`] loaded from `Tardy.toml`.
/// * The `windows` field holds a [`HashMap`] with keys of type [`window::WindowId`] and values of type [`Lens`].
impl App {
    /// Creates an instance of `App`.  Reads user key mappings from `Tardy.toml` using
    /// [`App::load_config`], then translates the mappings to commands using [`App::load_cmds`].
    pub fn new() -> Self {
        let mut app = Self::default();
        app.load_config();
        app.load_cmds();
        app
    }
    /// Instead of using a `WindowBuilder`, we now create a default instance of
    /// [`window::WindowAttributes`], and modify it to be transparent and carry the title `Tardy`.
    /// Besides looking cool, `winit` recommends setting the window to transparent if you are not
    /// ready to render anything to the window yet, to prevent "garbage capture".  As our program will terminate before we are
    /// ready, this is the ideal setting.
    ///
    /// We pass the attributes to the [`event_loop::ActiveEventLoop::create_window`] method.
    /// Here we wrap the window in an [`Arc`] for no good reason, though I would certainly *like*
    /// to need an [`Arc`] here to render an `egui` menu on top of a GIS map.
    ///
    /// Finally, we create an instance of [`Lens`] from the window, and insert it as a value into
    /// the [`HashMap`] in the `windows` field, using the window id as a key.
    ///
    /// The [polite]("https://github.com/crumplecup/cordial") crate is a shameless bit of laziness that
    /// I indulge in.  Primarily I use the [thiserror]("https://docs.rs/thiserror/latest/thiserror/")
    /// crate to bubble up errors from dependencies.  In new projects, I found myself copying the same
    /// code from project to project wherever they shared dependencies, just so I can use the question
    /// mark operator on them.  The `polite` library is an error-handling crate for shared
    /// dependencies among my own personal projects, and here it's a timesaver.  The error type for the
    /// library is called a `FauxPas`, and inclutes a `Hint(String)` variant that I often lean on to
    /// inject context when I don't want to stand up a full-fledged `MyCrateError` type, and yet
    /// I'm not ready to reach for [anyhow]("https://docs.rs/anyhow/latest/anyhow/").
    ///
    /// This method commits a `FauxPas` when [`event_loop::ActiveEventLoop::create_window`] fails.
    pub fn create_window(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
    ) -> polite::Polite<()> {
        let attr = window::Window::default_attributes()
            .with_title("Tardy")
            .with_transparent(true);
        let window = event_loop.create_window(attr)?;
        let window = Arc::new(window);
        // Did I create a window?
        tracing::trace!("Window created: {:?}", window.id());
        self.windows.insert(window.id(), Lens::new(window.clone()));
        // How many am I up to?
        tracing::trace!("Total windows: {}", self.windows.len());
        Ok(())
    }

    /// The user specifies key mappings in `Tardy.toml`, as described in the docs for [`Act`].
    /// I chose to use the [`config`] crate for parsing `toml`, as I'm likely to botch it if I
    /// tried to do it myself.  Here we call [`config::Config::builder`] and attempt to read in the
    /// source at `Tardy.toml`.
    ///
    /// If the build fails, we fall back on a default that happens to be exactly the same as my
    /// current `Tardy.toml`.  The current method has some drawbacks.  The default fallback would
    /// get onerous if I had more than two actions to worry about.  Also, I resort to unwrapping
    /// the default build, which will crash my program if it panics for some reason.
    pub fn load_config(&mut self) {
        if let Ok(config) = config::Config::builder()
            .add_source(config::File::with_name("Tardy"))
            .build()
        {
            self.config = config;
            // Sanity check that the file read correctly.
            tracing::trace!("Config set from file.");
        } else {
            // Warn me the user config couldn't be read.
            tracing::warn!("Could not read config from file.");
            let config = config::Config::builder();
            let config = config.set_default("exit", "Escape").unwrap();
            let config = config.set_default("new_window", "n").unwrap();
            let config = config.build().unwrap();
            self.config = config;
        }

        // Read the config to make sure its correct.
        tracing::trace!("{:#?}", self.config);
    }

    /// Keys and values play reversed roles in the [`Cmd`] and [`config::Config`] structs.  Here we
    /// convert one to the other using the [`Cmd::from`] implementation.
    /// Failure to read any commands from the config will produce an empty [`Cmd`], which will
    /// restrict the user to mouse interactions.
    pub fn load_cmds(&mut self) {
        let cmd = Cmd::from(&self.config);
        self.cmd = cmd;
        tracing::trace!("Commands read from config.");
        // Do you see the commands you expected?
        tracing::trace!("{:?}", self.cmd);
    }

    /// The act method dispatches program responses based upon the variant of [`Act`] passed in the
    /// `act` argument. Takes a mutable reference to `Self` in order to create and remove windows
    /// from the `windows` field.  The `id` parameter identifies the window upon which to apply the
    /// action.  The `event_loop` provides a reference to the active event loop for new window
    /// creation.
    ///
    /// We match on `act` and dispatch to the appropriate handler, before returning `Ok`.
    /// Commits a `FauxPas` when [`App::create_window`] fails.
    pub fn act(
        &mut self,
        act: &Act,
        id: &window::WindowId,
        event_loop: &event_loop::ActiveEventLoop,
    ) -> polite::Polite<()> {
        match act {
            Act::Exit => {
                tracing::trace!("Requesting exit.");
                let _ = self.windows.remove(id);
                Ok(())
            }
            Act::NewWindow => self.create_window(event_loop),
            Act::Be => {
                tracing::trace!("Taking it easy.");
                Ok(())
            }
        }
    }

    /// The `keyboard_input` method takes incoming keyboard presses and translates them to an [`Act`] variant using the [`Cmd::act`] method.
    /// If the key event passed in the `event` argument translates to a valid [`Act`], we pass it
    /// to the [`App::act`] method for handling.
    ///
    /// Takes a mutable reference to `self` to pass to [`App::act`].
    /// The `id` parameter indicates the active window and gets passed to [`App::act`].
    /// The `event_loop` parameter indicates the active event loop, and also gets passed to
    /// [`App::act`].
    /// Commits a `FauxPas` if [`App::act`] fails.
    pub fn keyboard_input(
        &mut self,
        id: &window::WindowId,
        event: &event::KeyEvent,
        event_loop: &event_loop::ActiveEventLoop,
    ) -> polite::Polite<()> {
        // Dispatch actions only on press.
        if event.state.is_pressed() {
            // Tell me I at least pressed the right key.
            tracing::trace!("Press detected: {:#?}", event);
            if let Some(act) = self.cmd.act(event) {
                // Helpful to know it triggered if the handler doesn't respond right.
                tracing::trace!("Act detected: {act}");
                self.act(&act, id, event_loop)?;
            } else {
                // No crime here.
                tracing::trace!("Invalid key.");
            }
        }
        Ok(())
    }
}

/// The impl for `ApplicationHandler` is boiled down to as little as possible.
/// * The `resumed` method gets called once at startup when the program is ready
///   to make the initial window.  Calls [`App::create_window`] and unwraps it with an `expect`.
/// * The `window_event` method removes the current window on a [`WindowEvent::CloseRequested`].
///   It dispatches keyboard input from a [`WindowEvent::KeyboardInput`] to the [`App::keyboard_input`]
///   method, converting errors to trace level logs (hopefully they weren't important).
/// * The [`WindowEvent::RedrawRequested`] variant will trigger a [`window::Window::request_redraw`]
///   call if the `refresh` field on [`Lens`] is set to `true`, which it never is.
/// * We delegate program exit to the `about_to_wait` method, where we check to see if there are open
///   windows remaining.  If all windows are closed, we exit gracefully.
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        self.create_window(event_loop)
            .expect("Could not create window.");
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        id: window::WindowId,
        event: WindowEvent,
    ) {
        let window = match self.windows.get_mut(&id) {
            Some(window) => window,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                tracing::trace!("Closing Window={id:?}");
                self.windows.remove(&id);
                tracing::trace!("Windows remaining: {}", self.windows.len());
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                match self.keyboard_input(&id, &event, event_loop) {
                    Ok(_) => tracing::trace!("Event handled."),
                    Err(e) => tracing::trace!("Unexpected: {}", e.to_string()),
                };
            }
            WindowEvent::RedrawRequested => {
                // I left these comments in from the example to remind me to put some cool stuff
                // here later.
                //
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                if *window.refresh() {
                    window.window().request_redraw();
                    window.with_refresh(false);
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        if self.windows.is_empty() {
            tracing::trace!("No windows left, exiting...");
            event_loop.exit();
        }
    }
}
