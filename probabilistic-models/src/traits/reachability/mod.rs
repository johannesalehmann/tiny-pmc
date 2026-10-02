mod backward;
pub use backward::BackwardReachability;

mod forward;
pub use forward::Reachability;

// TODO: This module is quite different from the other traits. Should it be moved elsewhere? Or
//  should the other traits be moved to the respective modules, this moved to operations and the
//  traits module scrapped entirely?
