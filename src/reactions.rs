use lazy_static::lazy_static;

use crate::materials::{Element, Reaction, State};

lazy_static! {
    pub static ref PROCESS_IRON_TO_GOLD: Reaction = Reaction {
        input: vec![Element::Iron.to_item_stack(State::Solid, 1)],
        output: vec![Element::Gold.to_item_stack(State::Solid, 1)],
    };
}
