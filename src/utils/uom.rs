// use uom::si::{electric_charge::coulomb, f32::*, f32::Quantity, mass::kilogram};
// use uom::si::rational;

use uom::{
    fmt::DisplayStyle,
    si::{Quantity, ISQ, SI},
    typenum::{N1, P1, Z0},
};

// // Create a `const` area of 1 square meter explicitly without using the `Area` alias.
// const A: Quantity<ISQ<P2, Z0, Z0, Z0, Z0, Z0, Z0>, SI<f32>, f32> =
//     Quantity { dimension: PhantomData, units: PhantomData, value: 1.0, };
// quantity! {
//     /// Mass to charge (base unit kilogram per coulomb, m · A⁻¹ · s⁻¹).
//     quantity: MassToCharge; "mass to charge";
//     /// Dimension of mass to charge, MI⁻¹T⁻¹ (base unit kilogram per coulomb, m · A⁻¹ · s⁻¹).
//     dimension: ISQ<
//         Z0,     // length
//         P1,     // mass
//         N1,     // time
//         N1,     // electric current
//         Z0,     // thermodynamic temperature
//         Z0,     // amount of substance
//         Z0>;    // luminous intensity
//     units {
//         /// Derived unit of mass to .
//         @kilogram_to_coulomb: prefix!(kilo) / prefix!(kilo); "kg/C", "kilogram to coulomb", "kilograms to coulombs";
//     }
// }

/// Определяем единицу измерения m/z в килограммах на кулон.
mod mass_to_charge {
    use uom::{
        si::{Quantity, Unit, ISQ, SI},
        typenum::{N1, P1, Z0},
    };

    // quantity! {
    //     /// Mass to charge (base unit kilogram per coulomb, m · A⁻¹ · s⁻¹).
    //     quantity: MassToCharge; "mass to charge";
    //     /// Dimension of mass to charge, MI⁻¹T⁻¹ (base unit kilogram per coulomb, m · A⁻¹ · s⁻¹).
    //     dimension: ISQ<
    //         Z0,     // length
    //         P1,     // mass
    //         N1,     // time
    //         N1,     // electric current
    //         Z0,     // thermodynamic temperature
    //         Z0,     // amount of substance
    //         Z0>;    // luminous intensity
    //     units {
    //         /// Derived unit of mass to .
    //         @kilogram_to_coulomb: prefix!(kilo) / prefix!(kilo); "kg/C", "kilogram to coulomb", "kilograms to coulombs";
    //     }
    // //   units {
    // //       #[cfg(feature = "uom")]
    // //       /// International System of Units (SI) unit of electric current, the ampere.
    // //       #[uom(ampere)]
    // //       /// Base unit of electric current in the International System of Units (SI), the ampere.
    // //     //   unitless: Prefix<P1, A>;
    // //       /// Coulombs per second.
    // //       #[cfg(feature = "uom")]
    // //       coulombs_per_second: (Prefix<P1, C>, Time^-1),
    // //       /// Coulombs per hour.
    // //       #[cfg(feature = "uom")]
    // //       coulombs_per_hour: (Prefix<P1, C>, Hour^-1),
    // //       // Добавление дополнительных единиц измерения возможно
    // //       // например, для измерения в миллиамперах:
    // //       milliamperes: Prefix<-3, A>;
    // //   }
    // }

    pub type MassToCharge = Quantity<ISQ<Z0, P1, N1, N1, Z0, Z0, Z0>, SI<f32>, f32>;
    // quantity! {
    //     units {
    //         /// Derived unit of mass to .
    //         @kilogram_to_coulomb: prefix!(kilo) / prefix!(kilo); "kg/C", "kilogram to coulomb", "kilograms to coulombs";
    //     }
    // }
}

// #[macro_use]
// mod time {
//     quantity! {
//         /// Time (base unit second, s).
//         quantity: Time; "time";
//         /// Time dimension, s.
//         dimension: Q<
//             Z0,  // length
//             Z0,  // mass
//             P1>; // time
//         units {
//             @second: 1.0; "s", "second", "seconds";
//         }
//     }
// }

// system! {
//     quantities: Q {
//         length: meter, L;
//         mass: kilogram, M;
//         time: second, T;
//     }

//     units: U {
//         mod length::Length,
//         mod mass::Mass,
//         mod time::Time,
//     }
// }

// mod f32 {
//     mod mks {
//         pub use super::super::*;
//     }

//     Q!(self::mks, f32);
// }

// unit! {
//     system: uom::si;
//     quantity: uom::si::mass;

//     @smoot: 1.702; "smoot", "smoot", "smoots";
// }

pub type MassToCharge = Quantity<ISQ<Z0, P1, N1, N1, Z0, Z0, Z0>, SI<f32>, f32>;

#[test]
fn test() {
    use uom::si::{electric_charge::coulomb, f32::*, mass::kilogram};
    // Пример использования типа m/z.
    // Создаем две массы и два заряда в килограммах и кулонах соответственно
    let mass_1 = Mass::new::<kilogram>(2.0);
    let mass_2 = Mass::new::<kilogram>(3.0);
    let charge_1 = ElectricCharge::new::<coulomb>(1.0);
    // let charge_2 = 2.0 * coulomb;

    // Вычисляем m/z для двух ионов
    let mz_1: MassToCharge = mass_1 / charge_1;
    let r: Ratio = mass_1 / Mass::new::<kilogram>(1.0);
    let r: Ratio = Ratio::from(3.0) / 20.0;
    // let mz_2 = (mass_2 / charge_2).get::<MassToCharge>();

    // Считаем разницу в m/z
    // let delta_mz = mz_1 - mz_2;

    // Выводим результат в консоль
    println!("ratio: {:?}", r.into_format_args(uom::si::ratio::ratio, DisplayStyle::Abbreviation));
    // println!(
    //     "m/z 1: {:?}",
    //     mz_1.into_format_args(kilogram / coulomb, DisplayStyle::Abbreviation)
    // );
    println!("m/z 1: {:?}", mass_1 / charge_1);
    // println!("m/z 2: {}", mz_2);
    // println!("Delta m/z: {}", delta_mz);
}
