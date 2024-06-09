build:
	cargo build --release

# example: make compile FILE_PATH=/tmp/main.rs
# in this case output file(s) will go in /tmp
compile: $(FILE_PATH)
	RUST_LOG=debug rustc -A warnings $< -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir $(dir $<)

compile_stdout: $(FILE_PATH)
	C_CODEGEN_COMMENTS=EXCLUDE rustc -A warnings -o - $< -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir $(dir $<)

# run tests with llvm's filecheck tool
test: tests/test_*.rs
	@total_tests=0; \
	passed_tests=0; \
	RUSTFLAGS='-Awarnings' cargo build --release --quiet; \
	\
	for file in $^; do \
	\
	\
		C_CODEGEN_COMMENTS=EXCLUDE rustc -A warnings -o - $$file -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir ./tests/ | cat | FileCheck-14 $$file; \
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

clean_tests:
	rm -f ./tests/*.c ./tests/*.h ./tests/e2e/*.out ./tests/e2e/*.o
	rm -f ./tests/e2e/*.c ./tests/e2e/*.h ./tests/e2e/*.out ./tests/e2e/*.o

clean:
	cargo clean

# for convenience sake
dummy:
	cargo build --release
	RUST_LOG=debug rustc ./tests/dummy.rs -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir ./tests/ 
