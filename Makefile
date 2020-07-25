.PHONY: upload clean
SSH := ${USER}@${IP}

target/armv7-unknown-linux-gnueabihf/debug/ACC: $(wildcard src/*.rs) Cargo.toml
	cargo build --target=armv7-unknown-linux-gnueabihf

upload: target/armv7-unknown-linux-gnueabihf/debug/ACC
	scp ./target/armv7-unknown-linux-gnueabihf/debug/ACC ${SSH}:/home/pi/acc

receive: upload
	ssh -t ${SSH} './acc receive'

clean:
	cargo clean
	rm -rf target || true
