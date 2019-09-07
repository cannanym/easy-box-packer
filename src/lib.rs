extern crate rutie;

use rutie::{Class, Object, Hash, Float, Fixnum, NilClass, Array, Symbol, AnyObject, VM};

rutie::class!(RustPacker);

fn to_dimension(rb_dimension: &AnyObject) -> f64 {
    match rb_dimension.try_convert_to::<Fixnum>() {
        Ok(i) => i.to_i64() as f64,
        Err(_) => rb_dimension.try_convert_to::<Float>().unwrap().to_f64()
    }
}

fn to_3d_array(rb_array : &AnyObject) -> [f64; 3] {
    let array = rb_array.try_convert_to::<Array>().unwrap();
    [
        to_dimension(&array.at(0)),
        to_dimension(&array.at(1)),
        to_dimension(&array.at(2)),
    ]
}

struct RotationAndMargin<'a> {
    rotation: &'a[f64; 3],
    smallest_margin: f64
}

rutie::methods!(
    RustPacker,
    _itself,

    fn place(item: Hash, space: Hash) -> AnyObject {
        let item_hash = item.unwrap();
        let space_hash = space.unwrap();
        let space_dimensions = to_3d_array(&space_hash.at(&Symbol::new("dimensions")));
        let mut item_dimensions = to_3d_array(&item_hash.at(&Symbol::new("dimensions")));
        item_dimensions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        item_dimensions.reverse();

        let permutations : [[f64; 3]; 6] = [
            [item_dimensions[1], item_dimensions[2], item_dimensions[0]],
            [item_dimensions[1], item_dimensions[0], item_dimensions[2]],
            [item_dimensions[2], item_dimensions[1], item_dimensions[0]],
            [item_dimensions[2], item_dimensions[0], item_dimensions[1]],
            [item_dimensions[0], item_dimensions[1], item_dimensions[2]],
            [item_dimensions[0], item_dimensions[2], item_dimensions[1]]
        ];

        let mut possible_rotations_and_margins : Vec<RotationAndMargin> = Vec::with_capacity(6);

        for rotation in permutations.iter() {
            if rotation[0] > space_dimensions[0] || rotation[1] > space_dimensions[1] || rotation[2] > space_dimensions[2] {
                continue;
            }
            let mut possible_margin = [
                space_dimensions[0] - rotation[0],
                space_dimensions[1] - rotation[1],
                space_dimensions[2] - rotation[2]
            ];
            let smallest_margin = possible_margin[0].min(possible_margin[1]).min(possible_margin[2]);
            possible_rotations_and_margins.push(
                RotationAndMargin { rotation, smallest_margin }
            );
        }

        if possible_rotations_and_margins.len() == 0 {
            return AnyObject::from(NilClass::new().value());
        }

        let mut result = Hash::new();

        possible_rotations_and_margins.sort_by(|a, b| a.smallest_margin.partial_cmp(&b.smallest_margin).unwrap());

        let mut rotation = Array::new();
        rotation.push(Float::new(possible_rotations_and_margins[0].rotation[0]));
        rotation.push(Float::new(possible_rotations_and_margins[0].rotation[1]));
        rotation.push(Float::new(possible_rotations_and_margins[0].rotation[2]));
        result.store(Symbol::new("dimensions"), rotation);
        result.store(Symbol::new("position"), space_hash.at(&Symbol::new("dimensions")));
        result.store(Symbol::new("weight"), item_hash.at(&Symbol::new("weight")));

        AnyObject::from(result.value())
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_rust_packer() {
    Class::new("RustPacker", None).define(|itself| {
        itself.def_self("place", place);
    });
}
