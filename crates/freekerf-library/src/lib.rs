//! FreeKerf Library: data model, validation, packaging and importers for the
//! open library of laser materials and machine profiles.
//!
//! Pipeline: [`loader::Loader`] reads `data/` into a [`loader::Library`],
//! [`validate::Validator`] runs semantic rules, [`package`] builds the release
//! archive and [`import`] converts third-party libraries.

pub mod diagnostics;
pub mod import;
pub mod layout;
pub mod loader;
pub mod model;
pub mod package;
pub mod safety;
pub mod schema;
pub mod text;
pub mod validate;

use diagnostics::Diagnostics;
use layout::Layout;
use loader::{Library, Loader};

/// Loads and validates the library at `layout` with the standard checks and rules.
pub fn check(layout: &Layout) -> (Library, Diagnostics) {
    let (library, mut diagnostics) = Loader::default().load(layout);
    diagnostics.extend(validate::Validator::standard().run(&validate::Context {
        library: &library,
        layout,
    }));
    diagnostics.sort();
    (library, diagnostics)
}
