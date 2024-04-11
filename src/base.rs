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
use rustc_middle::mir::BasicBlockData;
use rustc_middle::mir::StatementKind;
use rustc_middle::mir::{Place, Rvalue};
use rustc_middle::ty::print::with_no_trimmed_paths;
use rustc_session::config::{OutputFilenames, OutputType};
use std::io::Write;

pub struct Context {
    code: String,
}

impl Context {
    pub fn new() -> Self {
        Self { code: String::new() }
    }

    pub fn push(&mut self, code: &str) {
        self.code.push_str(code);
    }

    pub fn push_line(&mut self, code: &str) {
        self.code.push_str(code);
        self.code.push_str("\n");
    }

    pub fn get_code(&self) -> &str {
        &self.code
    }
}

pub struct OngoingCodegen {
    context: Context,
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

        let code = ongoing_codegen.context.get_code();
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(code.as_bytes()).unwrap();

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

#[allow(unused_variables)]
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
                let mir = tcx.instance_mir(inst.def);

                let blocks = &mir.basic_blocks;
                for (last_bb_id, block_data) in blocks.into_iter().enumerate() {
                    let block_data: &BasicBlockData = block_data;

                    let statements = &block_data.statements;
                    with_no_trimmed_paths!({
                        for stmt in statements {
                            writeln!(std::io::stdout(), "Statement: {:?}", stmt).unwrap();
                            writeln!(std::io::stdout(), "Statement Kind: {:?}", stmt.kind).unwrap();
                            match &stmt.kind {
                                StatementKind::Assign(val) => {
                                    let place = &val.0;
                                    let rvalue = &val.1;
                                    writeln!(std::io::stdout(), "Place: {:?}", place).unwrap();
                                    writeln!(std::io::stdout(), "Rvalue: {:?}", rvalue).unwrap();

                                    match rvalue {
                                        Rvalue::Repeat(operand, len) => {
                                            writeln!(std::io::stdout(), "Repeat",).unwrap();
                                        }
                                        Rvalue::Ref(a, b, c) => {
                                            writeln!(std::io::stdout(), "Ref",).unwrap();
                                        }
                                        Rvalue::ThreadLocalRef(region) => {
                                            writeln!(std::io::stdout(), "ThreadLocalRef",).unwrap();
                                        }
                                        Rvalue::AddressOf(a, b) => {
                                            writeln!(std::io::stdout(), "AddressOf",).unwrap();
                                        }
                                        Rvalue::Len(a) => {
                                            writeln!(std::io::stdout(), "Len",).unwrap();
                                        }
                                        Rvalue::Cast(kind, operand, ty) => {
                                            writeln!(std::io::stdout(), "Cast",).unwrap();
                                        }
                                        Rvalue::BinaryOp(op, operand1) => {
                                            writeln!(std::io::stdout(), "BinaryOp",).unwrap();
                                        }
                                        Rvalue::CheckedBinaryOp(op, operand1) => {
                                            writeln!(std::io::stdout(), "CheckedBinaryOp",)
                                                .unwrap();
                                        }
                                        Rvalue::NullaryOp(op, ty) => {
                                            writeln!(std::io::stdout(), "NullaryOp",).unwrap();
                                        }
                                        Rvalue::UnaryOp(op, operand) => {
                                            writeln!(std::io::stdout(), "UnaryOp",).unwrap();
                                        }
                                        Rvalue::Discriminant(place) => {
                                            writeln!(std::io::stdout(), "Discriminant",).unwrap();
                                        }
                                        Rvalue::Aggregate(kind, operands) => {
                                            writeln!(std::io::stdout(), "Aggregate",).unwrap();
                                        }
                                        Rvalue::ShallowInitBox(kind, operands) => {
                                            writeln!(std::io::stdout(), "ShallowInitBox",).unwrap();
                                        }
                                        Rvalue::CopyForDeref(kind) => {
                                            writeln!(std::io::stdout(), "CopyForDeref",).unwrap();
                                        }
                                        Rvalue::Use(operand) => match operand.constant() {
                                            Some(constant) => match constant.const_ {
                                                rustc_middle::mir::Const::Unevaluated(c, t) => {
                                                    writeln!(
                                                        std::io::stdout(),
                                                        "Const: {:?} {:?}",
                                                        tcx.const_eval_poly(c.def),
                                                        t
                                                    )
                                                    .unwrap();
                                                }
                                                _ => {}
                                            },
                                            None => {
                                                writeln!(std::io::stdout(), "Use: {:?}", operand)
                                                    .unwrap();
                                            }
                                        },
                                    }
                                }
                                _ => {}
                            }
                        }
                    });
                }

                with_no_trimmed_paths!({
                    let mut buf = Vec::new();

                    rustc_middle::mir::pretty::write_mir_fn(tcx, mir, &mut |_, _| Ok(()), &mut buf)
                        .unwrap();
                    ongoing_codegen.context.push_line(&String::from_utf8_lossy(&buf).into_owned());
                });
            }
            MonoItem::Static(def) => {
                ongoing_codegen.context.push_line(&format!("static {:?};", def));
            }
            MonoItem::GlobalAsm(item_id) => {
                ongoing_codegen
                    .context
                    .push_line(&format!("asm!(\"{:?}\";", tcx.hir().item(*item_id),));
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

    for cgu in &cgus {
        transpile_cgu(tcx, cgu, &mut ongoing_codegen);
    }

    let name: String = cgus.iter().next().unwrap().name().to_string();

    Box::new((name, ongoing_codegen, metadata, CrateInfo::new(tcx, "c".to_string())))
}
