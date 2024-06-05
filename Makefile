build:
	cargo build --release

# example: make compile FILE_PATH=/tmp/main.rs
# in this case output file(s) will go in /tmp
compile: $(FILE_PATH)
	rustc $< -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir $(dir $<)

test:
	rustc -C codegen-output=stdout ./tests/dummy.rs -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir ./tests/ 

clean:
	cargo clean

# for convenience sake
dummy:
	cargo build --release 
	rustc ./tests/dummy.rs -Z codegen-backend=./target/release/librustc_codegen_c.so --out-dir ./tests/ 