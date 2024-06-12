use crate::{
    definition::CVarDef,
    structure::{CComposite, CStructDef},
    ty::CType,
};

pub const FAT_PTR_NAME: &str = "codegenc_fat_ptr";
pub const FAT_PTR_DATA_FIELD: &str = "data";
pub const FAT_PTR_META_FIELD: &str = "meta";

pub fn new_fat_ptr() -> CComposite {
    let fat_ptr_composite = CComposite::Struct(CStructDef {
        name: FAT_PTR_NAME.to_string(),
        fields: vec![
            CVarDef::new(
                0,
                FAT_PTR_DATA_FIELD.to_string(),
                CType::Pointer(Box::new(CType::Void)),
            ),
            CVarDef::new(
                1,
                FAT_PTR_META_FIELD.to_string(),
                CType::Pointer(Box::new(CType::Void)),
            ),
        ],
    });

    fat_ptr_composite
}
