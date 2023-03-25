use bevy::{prelude::*, utils::hashbrown::HashMap};
use lazy_static::lazy_static;

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    pub input: Vec<ItemStack>,
    pub output: Vec<ItemStack>,
}

impl Reaction {
    pub fn valid_input(&self, input: &Vec<ItemStack>) -> bool {
        let matching = self
            .input
            .iter()
            .zip(input.iter())
            .filter(|&(rec, inp)| rec.item_type == inp.item_type && rec.quantity <= inp.quantity)
            .count();
        matching == self.input.len() && matching == input.len()
    }

    pub fn run(&self, input_inventory: &mut Vec<ItemStack>, output_inventory: &mut Vec<ItemStack>) {
        self.input.iter().for_each(|item| {
            input_inventory
                .iter_mut()
                .find(|i| i.item_type == item.item_type)
                .unwrap()
                .quantity -= item.quantity;
        });
        output_inventory.append(&mut self.output.clone());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item_type: ItemStackType,
    pub quantity: u32,
}

impl ItemStack {
    pub fn new(item_type: ItemStackType, quantity: u32) -> ItemStack {
        ItemStack {
            item_type,
            quantity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemStackType {
    Element(Element, State),
    Energy(Energy),
}

impl ItemStackType {
    pub fn to_item_stack(self, quantity: u32) -> ItemStack {
        ItemStack {
            item_type: self.clone(),
            quantity,
        }
    }

    pub fn quantity_limit(&self) -> u32 {
        ITEMSTACKTYPE_QUANTITY_LIMITS
            .get(self)
            .unwrap_or(&0)
            .clone()
    }
}

lazy_static! {
    pub static ref ITEMSTACKTYPE_QUANTITY_LIMITS: HashMap<ItemStackType, u32> =
        HashMap::from([(ItemStackType::Element(Element::Hydrogen, State::Solid), 100),]);
}

#[derive(Clone, Debug, PartialEq, Reflect, Eq, Hash)]
pub enum Energy {
    Mechanical,
    Electric,
    Magnetic,
    Gravitational,
    Chemical,
    Ionization,
    Nuclear,
    Chromodynamic,
    MechanicalWave,
    SoundWave,
    Radiant,
    Rest,
    Thermal,
}

impl Energy {
    pub fn to_item_stack(self, quantity: u32) -> ItemStack {
        ItemStack {
            item_type: ItemStackType::Energy(self),
            quantity,
        }
    }
}

// pub enum IonizingRadiation {
//     Ultraviolet,
//     Xray,
//     Gamma,
//     Alpha,
//     Beta,
//     Neutron,
// }
// pub enum NonIonizingRadiation {
//     UltravioletLight,
//     VisibleLight,
//     Infrared,
//     Microwave,
//     Radio,
//     Thermal,
//     Blackbody,
// }

#[derive(Clone, Debug, PartialEq, Reflect, Eq, Hash)]
pub enum State {
    Solid,
    Liquid,
    Gas,
    Plasma,
}

impl State {
    pub fn to_item_stack(self, element: Element, quantity: u32) -> ItemStack {
        ItemStack {
            item_type: ItemStackType::Element(element, self),
            quantity,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Reflect, Eq, Hash)]
pub enum Element {
    Hydrogen,
    Helium,
    Lithium,
    Beryllium,
    Boron,
    Carbon,
    Nitrogen,
    Oxygen,
    Fluorine,
    Neon,
    Sodium,
    Magnesium,
    Aluminium,
    Silicon,
    Phosphorus,
    Sulfur,
    Chlorine,
    Argon,
    Potassium,
    Calcium,
    Scandium,
    Titanium,
    Vanadium,
    Chromium,
    Manganese,
    Iron,
    Cobalt,
    Nickel,
    Copper,
    Zinc,
    Gallium,
    Germanium,
    Arsenic,
    Selenium,
    Bromine,
    Krypton,
    Rubidium,
    Strontium,
    Yttrium,
    Zirconium,
    Niobium,
    Molybdenum,
    Technetium,
    Ruthenium,
    Rhodium,
    Palladium,
    Silver,
    Cadmium,
    Indium,
    Tin,
    Antimony,
    Tellurium,
    Iodine,
    Xenon,
    Cesium,
    Barium,
    Lanthanum,
    Cerium,
    Praseodymium,
    Neodymium,
    Promethium,
    Samarium,
    Europium,
    Gadolinium,
    Terbium,
    Dysprosium,
    Holmium,
    Erbium,
    Thulium,
    Ytterbium,
    Lutetium,
    Hafnium,
    Tantalum,
    Tungsten,
    Rhenium,
    Osmium,
    Iridium,
    Platinum,
    Gold,
    Mercury,
    Thallium,
    Lead,
    Bismuth,
    Polonium,
    Astatine,
    Radon,
    Francium,
    Radium,
    Actinium,
    Thorium,
    Protactinium,
    Uranium,
    Neptunium,
    Plutonium,
    Americium,
    Curium,
    Berkelium,
    Californium,
    Einsteinium,
    Fermium,
    Mendelevium,
    Nobelium,
    Lawrencium,
    Rutherfordium,
    Dubnium,
    Seaborgium,
    Bohrium,
    Hassium,
    Meitnerium,
    Darmstadtium,
    Roentgenium,
    Copernicium,
    Nihonium,
    Flerovium,
    Moscovium,
    Livermorium,
    Tennessine,
    Oganesson,
}

impl Element {
    pub fn to_item_stack(self, state: State, quantity: u32) -> ItemStack {
        ItemStack {
            item_type: ItemStackType::Element(self, state),
            quantity,
        }
    }
}
