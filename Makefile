release: src/main.rs
	cargo build --release
	cp target/release/malti_split .
