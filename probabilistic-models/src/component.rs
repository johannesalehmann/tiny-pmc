// Marker trait that should be implemented by anything that slots into `Ini`, `ChLabel`, `BrLabel`,
// `Obs`, `APs`, `Rew`, `Ann`, `StateVals`, `Preds` that implements `Read...` (i.e. everything in
// those slots apart from `Option<...>` and `()`.
// It is used e.g. to concisely implement IntoOptionalComponent, which is used by
// Model::into_optional
pub trait Component {}

impl<T: Component> Component for &T {}
impl<T: Component> Component for &mut T {}
