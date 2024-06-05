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
use rustc_middle::ty::print::with_no_trimmed_paths;
use rustc_session::config::{CrateType, OutFileName, OutputFilenames, OutputType};
use rustc_session::output::out_filename;
use std::path::Path;
use tracing_subscriber::util::SubscriberInitExt;

use crate::function;
use crate::header;
use crate::include;
use crate::prefix;
use crate::structure::{self, CComposite, CStructDef};
use crate::ty::{CCompositeInfo, CType};
use crate::write;

pub struct Context {
    includes: Vec<include::Include>,
    header_includes: Vec<include::Include>,
    defines: Vec<header::CDefine>,
    functions: Vec<function::CFunction>,
    header_functions: Vec<function::CFunction>,
    structs: Vec<structure::CComposite>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            includes: Vec::new(),
            header_includes: Vec::new(),
            defines: Vec::new(),
            functions: Vec::new(),
            header_functions: Vec::new(),
            structs: Vec::new(),
        }
    }

    pub fn get_c_includes(&self) -> &Vec<include::Include> {
        &self.includes
    }

    pub fn get_mut_c_includes(&mut self) -> &mut Vec<include::Include> {
        &mut self.includes
    }
    pub fn get_h_includes(&self) -> &Vec<include::Include> {
        &self.header_includes
    }

    pub fn get_mut_h_includes(&mut self) -> &mut Vec<include::Include> {
        &mut self.header_includes
    }
    pub fn get_defines(&self) -> &Vec<header::CDefine> {
        &self.defines
    }
    pub fn get_mut_defines(&mut self) -> &mut Vec<header::CDefine> {
        &mut self.defines
    }

    pub fn get_functions(&self) -> &Vec<function::CFunction> {
        &self.functions
    }

    pub fn get_mut_functions(&mut self) -> &mut Vec<function::CFunction> {
        &mut self.functions
    }

    pub fn get_header_functions(&self) -> &Vec<function::CFunction> {
        &self.header_functions
    }

    pub fn get_mut_header_functions(&mut self) -> &mut Vec<function::CFunction> {
        &mut self.header_functions
    }

    pub fn get_structs(&self) -> &Vec<structure::CComposite> {
        &self.structs
    }

    pub fn get_mut_structs(&mut self) -> &mut Vec<structure::CComposite> {
        &mut self.structs
    }

    pub fn exists_header_fn_with_name(&self, name: &str) -> bool {
        for f in self.get_header_functions() {
            if f.get_name() == name {
                return true;
            }
        }
        return false;
    }

    pub fn add_composite(&mut self, composite: &CComposite) -> CCompositeInfo {
        self.get_mut_structs().push(composite.clone());

        let struct_idx = self.get_structs().len() - 1;

        let name = composite.get_name();

        return CCompositeInfo { name, ctx_idx: struct_idx };
    }

    pub fn get_composite(&self, info: &CCompositeInfo) -> CComposite {
        if self.get_structs().len() <= info.ctx_idx {
            panic!("Struct with index {} not found", info.ctx_idx);
        }

        return self.get_structs()[info.ctx_idx].clone();
    }

    pub fn get_struct_def(&self, info: &CCompositeInfo) -> Option<CStructDef> {
        match self.get_composite(info) {
            CComposite::Struct(s) | CComposite::Union(s) => return Some(s),
            _ => return None,
        }
    }

    pub fn get_field_name_for_struct(&self, info: &CCompositeInfo, idx: usize) -> Option<String> {
        if self.get_structs().len() <= info.ctx_idx {
            return None;
        }

        match &self.get_structs()[info.ctx_idx] {
            CComposite::Struct(s) | CComposite::Union(s) => {
                return Some(s.fields[idx].get_name().clone());
            }
            _ => return None,
        }
    }

    pub fn has_define_with_name(&self, name: &String) -> bool {
        for d in self.get_defines() {
            if d.get_name() == *name {
                return true;
            }
        }
        return false;
    }
}

pub struct OngoingCodegen {
    pub context: Context,
}

impl OngoingCodegen {
    pub fn join(
        &mut self,
        sess: &rustc_session::Session,
        name: String,
        metadata: EncodedMetadata,
        crate_info: CrateInfo,
        output_files: &OutputFilenames,
    ) -> CodegenResults {
        let output_name =
            out_filename(sess, CrateType::Executable, &output_files, crate_info.local_crate_name);

        let c_name = format!("{}.c", name);
        let h_name = format!("{}.h", name);
        let c_path: std::path::PathBuf =
            output_files.temp_path(OutputType::Object, Some(c_name.as_str()));
        let h_path = output_files.temp_path(OutputType::Object, Some(h_name.as_str()));

        let mut c_file = std::fs::File::create(&c_path).unwrap();
        let mut h_file = std::fs::File::create(&h_path).unwrap();

        match output_name {
            OutFileName::Real(path) => {
                self.context.get_mut_c_includes().push(include::Include::new(
                    format!(
                        "{}.h",
                        String::from(Path::new(&path).file_name().unwrap().to_str().unwrap()),
                    ),
                    false,
                ))
            }
            _ => (),
        }

        write::write_includes(
            self.context.get_c_includes(),
            self.context.get_h_includes(),
            &mut c_file,
            &mut h_file,
        );

        write::write_defines(self.context.get_defines(), &mut h_file);

        write::write_structs(self.context.get_structs(), &mut h_file);

        write::write_prototypes(self.context.get_functions(), &mut h_file);

        write::write_functions(self.context.get_functions(), &mut c_file, false);

        write::write_functions(self.context.get_header_functions(), &mut h_file, true);

        let modules = vec![
            CompiledModule {
                name: c_name,
                kind: rustc_codegen_ssa::ModuleKind::Regular,
                object: Some(c_path),
                bytecode: None,
                dwarf_object: None,
                assembly: None,
                llvm_ir: None,
            },
            CompiledModule {
                name: h_name,
                kind: rustc_codegen_ssa::ModuleKind::Metadata,
                object: Some(h_path),
                bytecode: None,
                dwarf_object: None,
                assembly: None,
                llvm_ir: None,
            },
        ];

        CodegenResults {
            crate_info: crate_info,
            modules: modules,
            allocator_module: None,
            metadata_module: None,
            metadata: metadata,
        }
    }
}

fn transpile_cgu<'tcx, 'ccx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    cgu: &CodegenUnit<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    rust_to_c_map: &'ccx mut std::collections::HashMap<rustc_middle::ty::Ty<'tcx>, CType>,
) {
    for (item, _data) in cgu.items() {
        if item.def_id().krate != 0u32.into() {
            continue;
        }

        match item {
            MonoItem::Fn(inst) => {
                with_no_trimmed_paths!({
                    function::handle_fn(tcx, ongoing_codegen, inst.clone(), rust_to_c_map);
                });
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
    let mut ongoing_codegen = Box::new(OngoingCodegen { context: Context::new() });
    let mut rust_to_c_map: std::collections::HashMap<rustc_middle::ty::Ty<'tcx>, CType> =
        std::collections::HashMap::new();

    tracing_subscriber::FmtSubscriber::builder()
        .with_line_number(true)
        .without_time()
        .with_max_level(tracing::Level::DEBUG)
        .finish()
        .init();

    // Build the prefix code
    prefix::build_prefix(&mut ongoing_codegen.context);

    for cgu in &cgus {
        transpile_cgu(tcx, cgu, &mut ongoing_codegen, &mut rust_to_c_map);
    }

    let name: String = cgus.iter().next().unwrap().name().to_string();

    Box::new((name, *ongoing_codegen, metadata, CrateInfo::new(tcx, "c".to_string())))
}
