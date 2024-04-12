extern crate rustc_abi;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_symbol_mangling;
extern crate rustc_target;
extern crate rustc_ty_utils;
extern crate stable_mir;

use core::panic;

use rustc_codegen_ssa::{CodegenResults, CompiledModule, CrateInfo};
use rustc_metadata::EncodedMetadata;
use rustc_middle::mir::mono::{CodegenUnit, MonoItem};
use rustc_session::config::{OutputFilenames, OutputType};

use crate::function;
use crate::include;
use crate::prefix;
use crate::structure;
use crate::write;

pub struct Context {
    includes: Vec<include::Include>,
    functions: Vec<function::CFunction>,
    structs: Vec<structure::CStruct>,
}

impl Context {
    pub fn new() -> Self {
        Self { includes: Vec::new(), functions: Vec::new(), structs: Vec::new() }
    }

    pub fn get_includes(&self) -> &Vec<include::Include> {
        &self.includes
    }

    pub fn get_mut_includes(&mut self) -> &mut Vec<include::Include> {
        &mut self.includes
    }

    pub fn get_functions(&self) -> &Vec<function::CFunction> {
        &self.functions
    }

    pub fn get_mut_functions(&mut self) -> &mut Vec<function::CFunction> {
        &mut self.functions
    }

    pub fn get_structs(&self) -> &Vec<structure::CStruct> {
        &self.structs
    }

    pub fn get_mut_structs(&mut self) -> &mut Vec<structure::CStruct> {
        &mut self.structs
    }
}

pub struct OngoingCodegen {
    pub context: Context,
}

impl OngoingCodegen {
    pub fn join(
        &self,
        name: String,
        ongoing_codegen: &OngoingCodegen,
        metadata: EncodedMetadata,
        crate_info: CrateInfo,
        output_files: &OutputFilenames,
    ) -> CodegenResults {
        let path = output_files.temp_path(OutputType::Object, Some(name.as_str()));

        let mut file = std::fs::File::create(&path).unwrap();

        write::write_includes(ongoing_codegen.context.get_includes(), &mut file);

        write::write_prototypes(ongoing_codegen.context.get_functions(), &mut file);

        write::write_structs(ongoing_codegen.context.get_structs(), &mut file);

        write::write_functions(ongoing_codegen.context.get_functions(), &mut file);

        let modules = vec![CompiledModule {
            name: name,
            kind: rustc_codegen_ssa::ModuleKind::Regular,
            object: Some(path),
            bytecode: None,
            dwarf_object: None,
            assembly: None,
            llvm_ir: None,
        }];

        CodegenResults {
            crate_info: crate_info,
            modules: modules,
            allocator_module: None,
            metadata_module: None,
            metadata: metadata,
        }
    }
}

fn transpile_cgu<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    cgu: &CodegenUnit<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
) {
    for (item, _data) in cgu.items() {
        if item.def_id().krate != 0u32.into() {
            continue;
        }

        match item {
            MonoItem::Fn(inst) => {
                function::handle_fn(tcx, ongoing_codegen, inst);
            }
            MonoItem::Static(def) => {
                panic!("Static items are not supported yet: {:?}", def);
            }
            MonoItem::GlobalAsm(item_id) => {
                panic!("Global asm items are not supported yet: {:?}", item_id);
            }
        }
    }
}

pub fn run<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    metadata: rustc_metadata::EncodedMetadata,
) -> Box<(String, OngoingCodegen, EncodedMetadata, CrateInfo)> {
    let cgus: Vec<_> = tcx.collect_and_partition_mono_items(()).1.iter().collect();
    let mut ongoing_codegen = OngoingCodegen { context: Context::new() };

    // Build the prefix code
    prefix::build_prefix(&mut ongoing_codegen.context);

    for cgu in &cgus {
        transpile_cgu(tcx, cgu, &mut ongoing_codegen);
    }

    let name: String = cgus.iter().next().unwrap().name().to_string();

    Box::new((name, ongoing_codegen, metadata, CrateInfo::new(tcx, "c".to_string())))
}
