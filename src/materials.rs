use bevy::prelude::*;

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
            .filter(|&(rec, inp)| rec.0 == inp.0 && rec.1 == inp.1 && rec.2 <= inp.2)
            .count();
        matching == self.input.len() && matching == input.len()
    }

    pub fn run(&self, input_inventory: &mut Vec<Item>, output_inventory: &mut Vec<Item>) {
        self.input.iter().for_each(|item| {
            input_inventory
                .iter_mut()
                .find(|i| i.0 == item.0 && i.1 == item.1)
                .unwrap()
                .2 -= item.2;
        });
        output_inventory.append(&mut self.output.clone());
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item(Option<Material>, Option<Energy>, f32);

#[derive(Clone, Debug, PartialEq)]
pub struct Material(Element, State);

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Solid,
    Liquid,
    Gas,
    Plasma,
}

#[derive(Clone, Debug, PartialEq)]
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
