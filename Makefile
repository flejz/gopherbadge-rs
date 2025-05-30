TARGET = thumbv6m-none-eabi
PROFILE ?= debug

ifeq (${PROFILE}, release)
	PROFILE_FLAG = --${PROFILE}
endif

build: dependencies
	cargo build --target ${TARGET} ${PROFILE_FLAG}

deploy: build
	cargo run --target ${TARGET} ${PROFILE_FLAG}

dev:
	cargo watch --clear -x "run --target ${TARGET} ${PROFILE_FLAG}"

check:
	cargo watch --clear -x "check --target ${TARGET}"

dependencies:
	@rustup target add ${TARGET}
	@rustup component add llvm-tools-preview
	@cargo install cargo-binutils
	@cargo install elf2uf2-rs