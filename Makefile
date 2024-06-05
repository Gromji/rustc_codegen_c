build:
	cargo build --release

# example: make compile FILE_PATH=/tmp/main.rs
# in this case output file(s) will go in /tmp
compile: $(FILE_PATH)
	rustc $< -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir $(dir $<)

compile_stdout: $(FILE_PATH)
	rustc -o - $< -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir $(dir $<)

# run tests with llvm's filecheck tool
test: tests/test_*.rs
	@total_tests=0; \
	passed_tests=0; \
	RUSTFLAGS='-Awarnings' cargo build --release --quiet; \
	\
	for file in $^; do \
	\
	\
		C_CODEGEN_COMMENTS=EXCLUDE rustc -A warnings -o - $$file -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir ./tests/ | cat | FileCheck-11 $$file; \
	\
	\
		if [ $$? -eq 0 ]; then \
			passed_tests=$$((passed_tests + 1)); \
		fi; \
		total_tests=$$((total_tests + 1)); \
	done; \
	\
	\
	echo "$$passed_tests/$$total_tests tests passed"; \
	\
	if [ $$passed_tests -ne $$total_tests ]; then \
		exit 1; \
	fi

clean:
	cargo clean

# for convenience sake
dummy:
	cargo build --release
	RUST_LOG=debug rustc ./tests/dummy.rs -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir ./tests/ 
