use bevy::prelude::*;

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    pub input: Vec<Item>,
    pub output: Vec<Item>,
}

impl Reaction {
    pub fn valid_input(&self, input: &Vec<Item>) -> bool {
        let matching = self
            .input
            .iter()
            .zip(input.iter())
            .filter(|&(rec, inp)| {
                rec.material == inp.material
                    && rec.energy == inp.energy
                    && rec.quantity <= inp.quantity
            })
            .count();
        matching == self.input.len() && matching == input.len()
    }

    pub fn run(&self, input_inventory: &mut Vec<Item>, output_inventory: &mut Vec<Item>) {
        self.input.iter().for_each(|item| {
            input_inventory
                .iter_mut()
                .find(|i| i.material == item.material && i.energy == item.energy)
                .unwrap()
                .quantity -= item.quantity;
        });
        output_inventory.append(&mut self.output.clone());
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub material: Option<Material>,
    pub energy: Option<Energy>,
    pub quantity: f32,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct Material {
    pub element: Element,
    pub state: State,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
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

#[derive(Clone, Debug, PartialEq, Reflect)]
pub enum State {
    Solid,
    Liquid,
    Gas,
    Plasma,
}

#[derive(Clone, Debug, PartialEq, Reflect)]
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
