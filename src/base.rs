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

use rustc_codegen_ssa::{CodegenResults, CompiledModule, CrateInfo};
use rustc_metadata::EncodedMetadata;
use rustc_middle::mir::mono::{CodegenUnit, MonoItem};
use rustc_session::config::{OutputFilenames, OutputType};

use crate::function;
use crate::prefix;
use crate::write;

pub struct Context {
    prefix_code: prefix::Prefix,
    functions: Vec<function::CFunction>,
}

impl Context {
    pub fn new() -> Self {
        Self { prefix_code: prefix::Prefix::new(), functions: Vec::new() }
    }

    pub fn get_mut_prefix(&mut self) -> &mut prefix::Prefix {
        &mut self.prefix_code
    }

    pub fn get_prefix(&self) -> &prefix::Prefix {
        &self.prefix_code
    }

    pub fn get_functions(&self) -> &Vec<function::CFunction> {
        &self.functions
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

        write::write_prefix(ongoing_codegen.context.get_prefix(), &mut file);

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
                ongoing_codegen.context.get_mut_prefix().push(&format!("static {:?};", def), true);
            }
            MonoItem::GlobalAsm(item_id) => {
                ongoing_codegen
                    .context
                    .get_mut_prefix()
                    .push(&format!("asm!(\"{:?}\";", tcx.hir().item(*item_id),), true);
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
    prefix::build_prefix(ongoing_codegen.context.get_mut_prefix());

    for cgu in &cgus {
        transpile_cgu(tcx, cgu, &mut ongoing_codegen);
    }

    let name: String = cgus.iter().next().unwrap().name().to_string();

    Box::new((name, ongoing_codegen, metadata, CrateInfo::new(tcx, "c".to_string())))
}
