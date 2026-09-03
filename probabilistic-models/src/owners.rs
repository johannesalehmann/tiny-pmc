// TODO: This is currently unused. It would be useful to implement ReadOwners for models that have
//  no owners. That way, algorithms can require M: ReadOwners<Type=SinglePlayer> to ensure that no
//  model with owners is passed in (which the algorithm might not be able to handle)
#[derive(Copy, Clone)]
pub enum SinglePlayer {
    SinglePlayer,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TwoPlayer {
    Eve,
    Adam,
}
