use crate::{objects::Objects, services::Service};

/// Implemented for objects that a partial object type exists for
pub trait HasPartial {
    /// The type representing the partial variant of this object
    type Partial: Partial<Full = Self>;

    /// Create an empty partial variant of this object
    ///
    /// This function exists just for convenience, and will just return a
    /// [`Default`] version of the partial object.
    fn partial() -> Self::Partial {
        Self::Partial::default()
    }

    /// Convert this object into its partial variant
    ///
    /// All fields of the partial variant are set from this object. This is
    /// useful when creating a new object that needs to share parts of an
    /// existing one.
    fn to_partial(&self) -> Self::Partial {
        self.into()
    }
}

/// Implemented for partial objects
///
/// The API for partial objects follows a specific style:
///
/// - Partial objects are structs with fields that mirror the fields of the full
///   object structs, but all fields are optional.
/// - Partial object structs have `with_*` methods to provide values for each of
///   their fields.
/// - Values provided to `with_*` are usually wrapped in an `Option`, and only a
///   `Some(...)` value has any effect. This is a trade-off that makes most use
///   cases slightly more verbose, while significantly simplifying more complex
///   use cases.
/// - Partial object structs may have other methods with prefixes like `as_*`,
///   `from_*`, or similar, if one or more of their fields can be initialized by
///   providing alternative data.
/// - Partial object structs have a `build` method to build a full object.
/// - All `with_*`, `as_*`, and `build` methods can be chained, to provide a
///   convenient API.
///
/// # Implementation Note
///
/// It would be nicer to require an [`Into`] bound instead of [`From`] (see
/// documentation of those types for more information). Unfortunately, this
/// doesn't seem to be possible without a `where` clause on `HasPartial`, which
/// significantly reduces the convenience of using that trait, as the `where`
/// clause needs to be repeated everywhere `HasPartial` is specialized as a
/// bound.
pub trait Partial: Default + for<'a> From<&'a Self::Full> {
    /// The type representing the full variant of this object
    type Full;

    /// Build a full object from this partial one
    ///
    /// Implementations of this method will typically try to infer any missing
    /// parts of the partial object, but this is not possible in all cases. In
    /// such cases, implementations of this method may panic.
    ///
    /// Calling `build` on a partial object that can't infer its missing parts
    /// is considered a programmer error, hence why this method doesn't return a
    /// [`Result`].
    fn build(self, objects: &mut Service<Objects>) -> Self::Full;
}
