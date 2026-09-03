#[derive(Copy, Clone)]
pub enum SinglePlayer {
    SinglePlayer,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TwoPlayer {
    Eve,
    Adam,
}
