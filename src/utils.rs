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
        Scalar::Int(i) => i.to_uint(i.size()),
        Scalar::Ptr(_, _) => panic!("Trying to get value of a pointer that is not supported!"),
    }
}

pub fn scalar_to_float(value: &Scalar) -> String {
    match value {
        Scalar::Int(i) =>{
            return match i.size().bytes() {
                2 => {
                    i.to_f16().to_string()
                }

                4 => {
                    i.to_f32().to_string()
                },

                8 => {
                    i.to_f64().to_string()
                },

                16  => {
                    i.to_f128().to_string()
                },
                
                _ => panic!("Unsupported float size!")
            }
        } 
        Scalar::Ptr(_, _) => panic!("Trying to get value of a pointer that is not supported!"),
    }
}

pub fn truncate_to_size(value: u128, bytes: usize) -> u128 {
    let mask = (1 << (bytes * 8)) - 1;
    value & mask
}