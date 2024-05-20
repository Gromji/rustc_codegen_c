use rustc_middle::mir::interpret::Scalar;
use rustc_middle::ty::Const;
pub fn const_to_usize(value: &Const) -> usize {
    const_to_u128(value).try_into().unwrap()
}

pub fn const_to_u128(value: &Const) -> u128 {
    let scalar = value.try_to_scalar().unwrap();
    scalar_to_u128(&scalar)
}

pub fn scalar_to_u128(value: &Scalar) -> u128 {
    match value {
        Scalar::Int(i) => i.try_to_uint(i.size()).unwrap(),
        Scalar::Ptr(_, _) => panic!("Trying to get value of a pointer that is not supported!"),
    }
}
