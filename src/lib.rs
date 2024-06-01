#![cfg_attr(doc, allow(internal_features))]
#![cfg_attr(doc, feature(rustdoc_internals))]
#![feature(rustc_private)]

extern crate rustc_codegen_ssa;
extern crate rustc_const_eval;
extern crate rustc_data_structures;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use base::OngoingCodegen;

use rustc_codegen_ssa::{traits::CodegenBackend, CrateInfo};

use rustc_data_structures::fx::FxIndexMap;
use rustc_metadata::EncodedMetadata;
use rustc_session::Session;
use std::{any::Any, path::Path};

mod aggregate;
mod base;
mod bb;
mod crepr;
mod definition;
mod expression;
mod function;
mod header;
mod include;
mod prefix;
mod stmt;
mod structure;
mod ty;
mod utils;
mod write;

// Maybe later change the way we name and include the header file.
const HEADER_FILE_NAME: &str = "header_file.h";

pub struct CCodegenBackend(());

impl CodegenBackend for CCodegenBackend {
    fn locale_resource(&self) -> &'static str {
        ""
    }

    // I think we can use this function to configure anything we want.
    // One way of doing it is to somehow use opts from sess (not sure how yet) to configure the backend
    // We can create a struct to hold the configuration and store it in CCodegenBackend
    fn init(&self, _sess: &Session) {}

    fn codegen_crate<'tcx>(
        &self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        metadata: rustc_metadata::EncodedMetadata,
        _need_metadata_module: bool,
    ) -> Box<dyn Any> {
        base::run(tcx, metadata)
    }

    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn Any>,
        _sess: &rustc_session::Session,
        outputs: &rustc_session::config::OutputFilenames,
    ) -> (
        rustc_codegen_ssa::CodegenResults,
        rustc_data_structures::fx::FxIndexMap<
            rustc_middle::dep_graph::WorkProductId,
            rustc_middle::dep_graph::WorkProduct,
        >,
    ) {
        let (name, ongoing_codegen, metadata, crate_info) = *ongoing_codegen
            .downcast::<(String, OngoingCodegen, EncodedMetadata, CrateInfo)>()
            .expect("in join_codegen: ongoing_codegen is not a OngoingCodegen");
        (
            ongoing_codegen.join(name, &ongoing_codegen, metadata, crate_info, outputs),
            FxIndexMap::default(),
        )
    }

    fn link(
        &self,
        sess: &rustc_session::Session,
        codegen_results: rustc_codegen_ssa::CodegenResults,
        outputs: &rustc_session::config::OutputFilenames,
    ) -> Result<(), rustc_span::ErrorGuaranteed> {
        use rustc_session::{
            config::{CrateType, OutFileName},
            output::out_filename,
        };
        use std::io::Write;
        let crate_name = codegen_results.crate_info.local_crate_name;
        let output_name = out_filename(sess, CrateType::Executable, &outputs, crate_name);
        match output_name {
            OutFileName::Real(ref path) => {
                let tmp_path = codegen_results.modules[0].object.as_ref().unwrap();
                let tmp_header_path = codegen_results.modules[1].object.as_ref().unwrap();

                // rename to out_file
                let tmp_path = Path::new(tmp_path);
                let tmp_header_path = Path::new(tmp_header_path);
                std::fs::rename(tmp_path, path.with_extension("c")).unwrap();
                std::fs::rename(tmp_header_path, path.with_file_name(HEADER_FILE_NAME)).unwrap();
            }
            OutFileName::Stdout => {
                let mut stdout = std::io::stdout();
                let tmp_path = codegen_results.modules[0].object.as_ref().unwrap();

                // print contents of tmp_path to stdout
                let tmp_path = Path::new(tmp_path);
                let contents = std::fs::read_to_string(tmp_path).unwrap();
                stdout.write_all(contents.as_bytes()).unwrap();

                // remove tmp_path
                std::fs::remove_file(tmp_path).unwrap();
            }
        }
        Ok(())
    }
}

// Calling the compiler with -Zcodegen-backend=<path to this codegen> will hot plug this backend
#[no_mangle]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(CCodegenBackend(()))
}
