use std::{fmt::Display, fmt::Formatter, time::Duration};

use bevy::{prelude::*, utils::hashbrown::HashMap};
use enum_iterator::Sequence;
use lazy_static::lazy_static;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Element>();
        app.register_type::<State>();
        app.register_type::<Reaction>();
        app.register_type::<ItemStack>();
        app.register_type::<ItemStackType>();
        app.register_type::<Energy>();
        app.register_type::<Inventory>();
    }
}

#[derive(Clone, Debug, PartialEq, Reflect, FromReflect, Default)]
pub struct Reaction {
    pub input: Vec<ItemStack>,
    pub output: Vec<ItemStack>,
    pub duration: Duration,
}

impl Display for Reaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for item in &self.input {
            write!(f, "{}", item)?;
        }
        write!(f, "-> ")?;
        for item in &self.output {
            write!(f, "{}", item)?;
        }
        write!(f, "({:?})", self.duration)
    }
}

impl Reaction {
    pub fn valid_input(&self, input: &Inventory) -> bool {
        if input.is_empty() {
            return false;
        }
        self.input.iter().all(|item| input.contains(item))
    }

    pub fn run(&self, input_inventory: &mut Inventory, output_inventory: &mut Inventory) {
        if input_inventory.is_empty() {
            return;
        }

        if !self.valid_input(input_inventory) {
            return;
        }

        self.input.iter().for_each(|ele| {
            input_inventory.remove(ele);
        });

        self.output.iter().for_each(|ele| {
            output_inventory.push(ele.clone());
        });
    }
}

#[derive(Debug, Clone, PartialEq, Reflect, FromReflect)]
pub struct ItemStack {
    pub item_type: ItemStackType,
    pub quantity: u32,
}

impl Display for ItemStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}),", self.item_type, self.quantity)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, FromReflect)]
pub enum ItemStackType {
    Element(Element, State),
    Energy(Energy),
}

impl Display for ItemStackType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
            .unwrap_or(&DEFAULT_STATIC_LIMIT)
            .clone()
    }
}

#[derive(Reflect, Default, Debug, Clone)]
pub struct Inventory {
    pub items: Vec<ItemStack>,
}

impl From<Vec<ItemStack>> for Inventory {
    fn from(items: Vec<ItemStack>) -> Self {
        Inventory { items }
    }
}

impl Inventory {
    pub fn contains(&self, filter: &ItemStack) -> bool {
        let total_local_quantity = self
            .items
            .iter()
            .filter_map(|item| {
                if item.item_type == filter.item_type {
                    Some(item.quantity)
                } else {
                    None
                }
            })
            .sum::<u32>();

        return total_local_quantity >= filter.quantity;
    }
    pub fn transfer(&mut self, requested: &ItemStack, destination: &mut Inventory) {
        let total_local_quantity = self
            .items
            .iter()
            .filter_map(|item| {
                if item.item_type == requested.item_type {
                    Some(item.quantity)
                } else {
                    None
                }
            })
            .sum::<u32>();

        if total_local_quantity < requested.quantity {
            return;
        }

        let mut amount_left_to_take: u32 = requested.quantity;

        for item in self.items.iter_mut() {
            if amount_left_to_take == 0 {
                break;
            }
            if item.item_type != requested.item_type || item.quantity == 0 {
                continue;
            }
            if item.quantity > amount_left_to_take {
                item.quantity -= amount_left_to_take;
                destination.push(ItemStack {
                    item_type: item.item_type.clone(),
                    quantity: amount_left_to_take,
                });
                amount_left_to_take = 0;
            } else if item.quantity < amount_left_to_take {
                destination.push(item.clone());
                amount_left_to_take -= item.quantity;
                item.quantity = 0;
            } else {
                destination.push(item.clone());
                amount_left_to_take -= item.quantity;
                item.quantity = 0;
            }
        }

        self.items.retain(|item| item.quantity > 0);
    }

    pub fn transfer_first(&mut self, destination: &mut Inventory) {
        if self.items.is_empty() {
            return;
        }
        let item = self.items.remove(0);
        destination.push(item);
    }

    pub fn push(&mut self, item: ItemStack) {
        let mut amount_left_to_add: u32 = item.quantity;

        for stack in self.items.iter_mut() {
            if amount_left_to_add == 0 {
                break;
            }
            if stack.item_type != item.item_type {
                continue;
            }
            if stack.quantity + amount_left_to_add < stack.item_type.quantity_limit() {
                stack.quantity += amount_left_to_add;
                amount_left_to_add = 0;
            } else if stack.quantity + amount_left_to_add > stack.item_type.quantity_limit() {
                amount_left_to_add -= stack.item_type.quantity_limit() - stack.quantity;
                stack.quantity = stack.item_type.quantity_limit();
            } else {
                amount_left_to_add = 0;
                stack.quantity = stack.item_type.quantity_limit();
            }
        }

        if amount_left_to_add == 0 {
            return;
        }

        while amount_left_to_add > 0 {
            if amount_left_to_add < item.item_type.quantity_limit() {
                self.items.push(ItemStack {
                    item_type: item.item_type.clone(),
                    quantity: amount_left_to_add,
                });
                break;
            }
            self.items.push(ItemStack {
                item_type: item.item_type.clone(),
                quantity: item.item_type.quantity_limit(),
            });
            amount_left_to_add -= item.item_type.quantity_limit();
        }
    }

    pub fn pop(&mut self) -> Option<ItemStack> {
        self.items.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn remove(&mut self, item: &ItemStack) {
        let mut amount_left_to_take: u32 = item.quantity;

        if amount_left_to_take == 0 {
            return;
        }

        for stack in self.items.iter_mut() {
            if stack.item_type != item.item_type || stack.quantity == 0 {
                continue;
            }
            if stack.quantity > amount_left_to_take {
                stack.quantity -= amount_left_to_take;
                amount_left_to_take = 0;
            } else if stack.quantity < amount_left_to_take {
                amount_left_to_take -= stack.quantity;
                stack.quantity = 0;
            } else {
                amount_left_to_take -= stack.quantity;
                stack.quantity = 0;
            }
        }

        self.items.retain(|item| item.quantity > 0);
    }
}

lazy_static! {
    pub static ref ITEMSTACKTYPE_QUANTITY_LIMITS: HashMap<ItemStackType, u32> =
        HashMap::from([(ItemStackType::Element(Element::Hydrogen, State::Solid), 100)]);
    pub static ref DEFAULT_STATIC_LIMIT: u32 = 64;
}

#[derive(Clone, Debug, PartialEq, Reflect, Eq, Hash, FromReflect, Sequence, Default)]
pub enum Energy {
    #[default]
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

impl Display for Energy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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

#[derive(Clone, Debug, PartialEq, Reflect, Eq, Hash, FromReflect, Sequence, Default)]
pub enum State {
    #[default]
    Solid,
    Liquid,
    Gas,
    Plasma,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl State {
    pub fn to_item_stack(self, element: Element, quantity: u32) -> ItemStack {
        ItemStack {
            item_type: ItemStackType::Element(element, self),
            quantity,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Reflect, Eq, Hash, FromReflect, Sequence, Default)]
pub enum Element {
    #[default]
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

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
