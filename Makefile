.sim:
	cargo build
	mv target/debug/cache-simulator sim

clean:
	cargo clean