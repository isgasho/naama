CARGO:=cargo.exe +nightly

VST_LIB:=libs/vst-rs

ifeq ($(PROFILE),release)
	CARGO_FLAGS:=$(CARGO_FLAGS)" --release"
else
	PROFILE=debug
endif

ifeq ($(CARGO_CMD),)
	CARGO_CMD="build"
endif


all:
	${CARGO} ${CARGO_CMD} ${CARGO_FLAGS}

host:
	${CARGO} ${CARGO_CMD} ${CARGO_FLAGS} --bin host

example-vst:
	cd ${VST_LIB} && ${CARGO} build --examples
	cp ${VST_LIB}/target/${PROFILE}/examples/gain_effect.dll ./examples/vst
	
.PHONY: all host