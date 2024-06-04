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
use rustc_session::config::{OutputFilenames, OutputType};
use tracing_subscriber::util::SubscriberInitExt;

use crate::function;
use crate::header;
use crate::include;
use crate::prefix;
use crate::structure;
use crate::ty::CType;
use crate::write;

pub struct Context {
    includes: Vec<include::Include>,
    header_includes: Vec<include::Include>,
    defines: Vec<header::CDefine>,
    functions: Vec<function::CFunction>,
    header_functions: Vec<function::CFunction>,
    structs: Vec<structure::CStruct>,
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

    pub fn get_includes(&self) -> &Vec<include::Include> {
        &self.includes
    }

    pub fn get_mut_includes(&mut self) -> &mut Vec<include::Include> {
        &mut self.includes
    }
    pub fn get_header_includes(&self) -> &Vec<include::Include> {
        &self.header_includes
    }

    pub fn get_mut_header_includes(&mut self) -> &mut Vec<include::Include> {
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

    pub fn get_structs(&self) -> &Vec<structure::CStruct> {
        &self.structs
    }

    pub fn get_mut_structs(&mut self) -> &mut Vec<structure::CStruct> {
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

    /// Get the name of a struct that has the same list of types, or create one if it doesn't exist.
    pub fn get_struct(&mut self, list: &Vec<CType>) -> structure::CStruct {
        let cur_struct = structure::CStruct::from(list);
        let structs = self.get_structs();
        for s in structs {
            if s == &cur_struct {
                return s.clone();
            }
        }
        // Struct doesn't exist, create it
        self.get_mut_structs().push(cur_struct.clone());
        return cur_struct;
    }

    pub fn get_field_name_for_struct(&self, struct_name: &str, idx: usize) -> Option<String> {
        for s in self.get_structs() {
            if s.get_name() == struct_name {
                return Some(s.get_field(idx).get_name().clone());
            }
        }
        
        return None;
    }

    pub fn has_struct_with_name(&self, name: &str) -> bool {
        for s in self.get_structs() {
            if s.get_name() == name {
                return true;
            }
        }
        return false;
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
        &self,
        name: String,
        ongoing_codegen: &OngoingCodegen,
        metadata: EncodedMetadata,
        crate_info: CrateInfo,
        output_files: &OutputFilenames,
    ) -> CodegenResults {
        let path = output_files.temp_path(OutputType::Object, Some(name.as_str()));
        let header_name = format!("{}_h", name);
        let header_path = output_files.temp_path(OutputType::Object, Some(header_name.as_str()));

        let mut file = std::fs::File::create(&path).unwrap();
        let mut header_file = std::fs::File::create(&header_path).unwrap();

        write::write_includes(
            ongoing_codegen.context.get_includes(),
            ongoing_codegen.context.get_header_includes(),
            &mut file,
            &mut header_file,
        );

        write::write_defines(ongoing_codegen.context.get_defines(), &mut header_file);

        write::write_structs(ongoing_codegen.context.get_structs(), &mut header_file);
        
        write::write_prototypes(ongoing_codegen.context.get_functions(), &mut header_file);

        write::write_functions(ongoing_codegen.context.get_functions(), &mut file, false);

        write::write_functions(
            ongoing_codegen.context.get_header_functions(),
            &mut header_file,
            true,
        );

        let modules = vec![
            CompiledModule {
                name: name,
                kind: rustc_codegen_ssa::ModuleKind::Regular,
                object: Some(path),
                bytecode: None,
                dwarf_object: None,
                assembly: None,
                llvm_ir: None,
            },
            CompiledModule {
                name: header_name,
                kind: rustc_codegen_ssa::ModuleKind::Metadata,
                object: Some(header_path),
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
                with_no_trimmed_paths!({
                    function::handle_fn(tcx, ongoing_codegen, inst.clone());
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

    tracing_subscriber::FmtSubscriber::builder()
        .with_line_number(true)
        .without_time()
        .with_max_level(tracing::Level::DEBUG)
        .finish()
        .init();

    // Build the prefix code
    prefix::build_prefix(&mut ongoing_codegen.context);

    for cgu in &cgus {
        transpile_cgu(tcx, cgu, &mut ongoing_codegen);
    }

    let name: String = cgus.iter().next().unwrap().name().to_string();

    Box::new((name, *ongoing_codegen, metadata, CrateInfo::new(tcx, "c".to_string())))
}
