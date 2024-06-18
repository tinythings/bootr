.DEFAULT_GOAL := build
.PHONY:build bootr-release-static bootr-debug-static

ARCH := $(shell uname -p)
ARC_VERSION := $(shell cat src/bootr.rs | grep 'static VERSION' | sed -e 's/.*=//g' -e 's/[" ;]//g')
ARC_NAME := bootr-${ARC_VERSION}

bootr-release-static:
	RUSTFLAGS='-C target-feature=+crt-static' cargo build -p bootr --target $(ARCH)-unknown-linux-gnu --release

bootr-debug-static:
	RUSTFLAGS='-C target-feature=+crt-static' cargo build -p bootr --target $(ARCH)-unknown-linux-gnu

build-debug:
	@printf "Building Bootr (debug)\n"
	@$(MAKE) bootr-debug-static
	@printf "\n\nDone. Debug version is built for you in target/debug\n\n"

build-release:
	@printf "Building Bootr (release)\n"
	@$(MAKE) bootr-release-static
	@printf "\n\nDone. Debug version is built for you in target/release\n\n"

tar:
	rm -rf package/${ARC_NAME}
	cargo vendor
	mkdir -p package/${ARC_NAME}/.cargo
	cp .vendor.toml package/${ARC_NAME}/.cargo/config.toml

	cp LICENSE package/${ARC_NAME}
	cp README.md package/${ARC_NAME}
	cp Cargo.lock package/${ARC_NAME}
	cp Cargo.toml package/${ARC_NAME}
	cp Makefile package/${ARC_NAME}
	cp -a src package/${ARC_NAME}
	cp -a vendor package/${ARC_NAME}

	# Cleanup. Also https://github.com/rust-lang/cargo/issues/7058
	find package/${ARC_NAME} -type d -wholename "*/target" -prune -exec rm -rf {} \;
	find package/${ARC_NAME} -type d -wholename "*/vendor/winapi*" -prune -exec \
		rm -rf {}/src \; -exec mkdir -p {}/src \; -exec touch {}/src/lib.rs \; -exec rm -rf {}/lib \;
	find package/${ARC_NAME} -type d -wholename "*/vendor/windows*" -prune -exec \
		rm -rf {}/src \; -exec mkdir -p {}/src \;  -exec touch {}/src/lib.rs \; -exec rm -rf {}/lib \;
	rm -rf package/${ARC_NAME}/vendor/web-sys/src/*
	rm -rf package/${ARC_NAME}/vendor/web-sys/webidls
	mkdir -p package/${ARC_NAME}/vendor/web-sys/src
	touch package/${ARC_NAME}/vendor/web-sys/src/lib.rs

	# Tar the source
	tar -C package -czvf package/${ARC_NAME}.tar.gz ${ARC_NAME}
	rm -rf package/${ARC_NAME}
	rm -rf vendor
