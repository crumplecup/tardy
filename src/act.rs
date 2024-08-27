use convert_case::Casing;

/// The `act` module provides the `Act` enum for encapsulating action handling.
/// We match keyboard events to variants of the `Act` enum, then dispatch events handlers by matching against the `Act` enum.
///
/// # Dispatching actions with `Act`
///
/// The `Act` enum contains a variant for each action accessible to the user.  The purpose of this
/// enum is to allow the application to identify user intent encapsulated by the enum variant, then
/// dispatch an action based upon the context of the application state.
///
/// Currently we match incoming keyboard events to variants of `Act` using the [`crate::Cmd`]
/// struct.  The user determines the mapping of variants to keyboard input by specifying the
/// relationship using a \[key = "value"\] syntax in the `Tardy.toml` file.  The `key` is the
/// variant name in snake case, and `value` is the desired key mapping.
///
/// Upon initial load, the application will attempt to read valid key mappings from `Tardy.toml`
/// into a [`config::Config`] struct contained in the `config` field of [`crate::App`],
/// and will warn the user if no mappings return and substitute a default configuration instead.
///
/// Modifers are not currently supported, so only use single characters as `value` arguments.
///
/// ## Update 0.1.1
///
/// The `Act` enum now includes a `CloseWindow` variant, indicating the user intent to close the
/// window.  Once we successfully created background processes to spawn new windows, the need to
/// subsequently close windows became clear.
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum_macros::EnumIter,
    derive_more::Display,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Act {
    /// The `CloseWindow` variant indicates the user would like to close the current window.
    CloseWindow,
    /// The `Exit` variant indicates the user would like to close the current window.
    Exit,
    /// The `NewWindow` variant indicates the user would like to create a new window.
    NewWindow,
    /// The `Be` variant does nothing.
    #[default]
    Be,
}

/// We use [`derive_more::Display`] to derive [`std::fmt::Display`], with the understanding that
/// the corresponding implementation of `to_string()` will produce CamelCase.
/// The [`convert_case`] crate allows us to switch to other cases as needed.
///
/// This is an example where the benefit to developer experience is worth the burdern of extra
/// dependencies ([`derive_more`] and [`convert_case`]), as well as longer compile times and some binary bloat (after
/// all, I have no plans to use `to_string()` as anything more than an intermediatiary and
/// could just match directly.  But the resulting code is so clean, it is easy to write,
/// understand, and hopefully maintain, and that is worth the trade-off to me.)
impl Act {
    /// The `title` method converts the variant name to `Title` case using
    /// [`convert_case::Case::Title`].
    /// Use this method to generate a display name for user consumption.
    pub fn title(&self) -> String {
        self.to_string().to_case(convert_case::Case::Title)
    }

    /// The `snake` method converts the variant name to `Snake` case using
    /// [`convert_case::Case::Snake`].
    /// Used to read variant names as keys in `Tardy.toml`.  We check against each variant of
    /// `Act` to see if the matching snake case key is present in the toml file.
    pub fn snake(&self) -> String {
        self.to_string().to_case(convert_case::Case::Snake)
    }
}
