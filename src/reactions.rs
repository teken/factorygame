use crate::materials::{Element, Item, Material, Reaction, State};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROCESS_IRON_TO_GOLD: Reaction = Reaction {
        input: vec![Item {
            material: Some(Material {
                element: Element::Iron,
                state: State::Solid,
            }),
            energy: None,
            quantity: 1.0,
        }],
        output: vec![Item {
            material: Some(Material {
                element: Element::Gold,
                state: State::Solid,
            }),
            energy: None,
            quantity: 1.0,
        }],
    };
}
