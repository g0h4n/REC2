prog :=rec2
server :=server
implant :=rec2
#LITCRYPT_ENCRYPT_KEY to change
masterkey :=RAMDOMdd28f0dcd9779315ee130deb565dbf315587f1611e54PASSWORD

cargo := $(shell command -v cargo 2> /dev/null)
cargo_v := $(shell cargo -V| cut -d ' ' -f 2)
rustup := $(shell command -v rustup 2> /dev/null)

check_cargo:
  ifndef cargo
    $(error cargo is not available, please install it! curl https://sh.rustup.rs -sSf | sh)
  else
	@echo "Make sure your cargo version is up to date! Current version is $(cargo_v)"
  endif

check_rustup:
  ifndef rustup
    $(error rustup is not available, please install it! curl https://sh.rustup.rs -sSf | sh)
  endif

# Deps install

install_windows_deps: update_rustup
	@rustup install stable-x86_64-pc-windows-gnu --force-non-host
	@rustup target add x86_64-pc-windows-gnu
	@rustup install stable-i686-pc-windows-gnu --force-non-host
	@rustup target add i686-pc-windows-gnu

install_macos_deps:
	@sudo git clone https://github.com/tpoechtrager/osxcross /usr/local/bin/osxcross || exit
	@sudo wget -P /usr/local/bin/osxcross/ -nc https://s3.dockerproject.org/darwin/v2/MacOSX10.10.sdk.tar.xz && sudo mv /usr/local/bin/osxcross/MacOSX10.10.sdk.tar.xz /usr/local/bin/osxcross/tarballs/
	@sudo UNATTENDED=yes OSX_VERSION_MIN=10.7 /usr/local/bin/osxcross/build.sh
	@sudo chmod 775 /usr/local/bin/osxcross/ -R
	@export PATH="/usr/local/bin/osxcross/target/bin:$PATH"
	@grep 'target.x86_64-apple-darwin' ~/.cargo/config || echo "[target.x86_64-apple-darwin]" >> ~/.cargo/config
	@grep 'linker = "x86_64-apple-darwin14-clang"' ~/.cargo/config || echo 'linker = "x86_64-apple-darwin14-clang"' >> ~/.cargo/config
	@grep 'ar = "x86_64-apple-darwin14-clang"' ~/.cargo/config || echo 'ar = "x86_64-apple-darwin14-clang"' >> ~/.cargo/config

install_linux_deps:update_rustup
	@rustup install stable-x86_64-unknown-linux-gnu --force-non-host
	@rustup target add x86_64-unknown-linux-gnu

install_cross:
	@cargo install --version 0.1.16 cross

update_rustup:
	rustup update

# Cleaning

clean:
	sudo rm -rf server/target implants/*/target

# Makefile for C2 server

c2server_release: check_cargo
	cargo build --release --manifest-path server/Cargo.toml
	cp server/target/release/$(server) ./$(server)_release
	@echo -e "[+] You can find \033[1;32m$(server)_release\033[0m release version in your current folder."

c2server_debug: check_cargo
	cargo build --manifest-path server/Cargo.toml
	cp server/target/debug/$(server) ./$(server)_debug
	@echo -e "[+] You can find \033[1;32m$(server)_debug\033[0m debug version in your current folder."

c2server_doc: check_cargo
	cargo doc --open --no-deps --manifest-path server/Cargo.toml

c2server_build_windows_x64:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-pc-windows-gnu --manifest-path  server/Cargo.toml
	cp server/target/x86_64-pc-windows-gnu/release/$(server).exe ./$(server)_x64.exe
	@echo -e "[+] You can find \033[1;32m$(server)_x64.exe\033[0m in your current folder."

c2server_build_windows_x86:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target i686-pc-windows-gnu --manifest-path server/Cargo.toml
	cp server/target/i686-pc-windows-gnu/release/$(server).exe ./$(server)_x86.exe
	@echo -e "[+] You can find \033[1;32m$(server)_x86.exe\033[0m in your current folder."

c2server_windows: check_rustup install_windows_deps c2server_build_windows_x64

c2server_windows_x64: check_rustup install_windows_deps c2server_build_windows_x64

c2server_windows_x86: check_rustup install_windows_deps c2server_build_windows_x86

c2server_build_linux_aarch64:
	cross build --target aarch64-unknown-linux-gnu --release --manifest-path server/Cargo.toml
	cp server/target/aarch64-unknown-linux-gnu/release/$(server) ./$(server)_aarch64
	@echo -e "[+] You can find \033[1;32m$(server)_aarch64\033[0m in your current folder."

c2server_linux_aarch64: check_rustup install_cross c2server_build_linux_aarch64

c2server_build_linux_x86_64:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu --manifest-path server/Cargo.toml
	cp server/target/x86_64-unknown-linux-gnu/release/$(server) ./$(server)_x86_64
	@echo -e "[+] You can find \033[1;32m$(server)_x86_64\033[0m in your current folder."

c2server_linux_x86_64: check_rustup install_linux_deps c2server_build_linux_x86_64

c2server_linux: check_rustup install_linux_deps c2server_build_linux_x86_64

c2server_build_macos:
	@export PATH="/usr/local/bin/osxcross/target/bin:$PATH"
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-apple-darwin --manifest-path server/Cargo.toml
	cp server/target/x86_64-apple-darwin/release/$(server) ./$(server)_macOS
	@echo -e "[+] You can find \033[1;32m$(server)_macOS\033[0m in your current folder."

c2server_macos: check_rustup install_cross install_macos_deps c2server_build_macos

c2server_arm_musl: check_rustup install_cross
	cross build --target arm-unknown-linux-musleabi --release
	cp server/target/arm-unknown-linux-musleabi/release/$(server) ./$(server)_arm_musl
	@echo -e "[+] You can find \033[1;32m$(server)_arm_musl\033[0m in your current folder."

c2server_armv7: check_rustup install_cross
	cross build --target armv7-unknown-linux-gnueabihf --release
	cp server/target/armv7-unknown-linux-gnueabihf/release/$(server) ./$(server)_armv7
	@echo -e "[+] You can find \033[1;32m$(server)_armv7\033[0m in your current folder."

# Makefile for Mastodon implant

mastodon_release: check_cargo
	cargo build --release --manifest-path implants/mastodon/Cargo.toml
	cp implants/mastodon/target/release/$(prog) ./$(prog)_mastodon_release
	@echo -e "[+] You can find \033[1;32m$(prog)_mastodon_release\033[0m release version in your current folder."

mastodon_debug: check_cargo
	cargo build --manifest-path implants/mastodon/Cargo.toml
	cp implants/mastodon/target/debug/$(prog) ./$(prog)_mastodon_debug
	@echo -e "[+] You can find \033[1;32m$(prog)_mastodon_debug\033[0m debug version in your current folder."

mastodon_doc: check_cargo
	cargo doc --open --no-deps --manifest-path implants/mastodon/Cargo.toml

mastodon_build_windows_x64:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-pc-windows-gnu --manifest-path  implants/mastodon/Cargo.toml
	cp implants/mastodon/target/x86_64-pc-windows-gnu/release/$(prog).exe ./$(prog)_mastodon_x64.exe
	@echo -e "[+] You can find \033[1;32m$(prog)_mastodon_x64.exe\033[0m in your current folder."

mastodon_build_windows_x86:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target i686-pc-windows-gnu --manifest-path implants/mastodon/Cargo.toml
	cp implants/mastodon/target/i686-pc-windows-gnu/release/$(prog).exe ./$(prog)_mastodon_x86.exe
	@echo -e "[+] You can find \033[1;32m$(prog)_mastodon_x86.exe\033[0m in your current folder."

mastodon_windows: check_rustup install_windows_deps replace_key mastodon_build_windows_x64 gen_key

mastodon_windows_x64: check_rustup install_windows_deps replace_key mastodon_build_windows_x64 gen_key

mastodon_windows_x86: check_rustup install_windows_deps replace_key mastodon_build_windows_x86 gen_key

mastodon_build_linux_aarch64:
	cross build --target aarch64-unknown-linux-gnu --release --manifest-path implants/mastodon/Cargo.toml
	cp implants/mastodon/target/aarch64-unknown-linux-gnu/release/$(prog) ./$(prog)_mastodon_aarch64
	@echo -e "[+] You can find \033[1;32m$(prog)_mastodon_aarch64\033[0m in your current folder."

mastodon_linux_aarch64: check_rustup install_cross replace_key mastodon_build_linux_aarch64 gen_key

mastodon_build_linux_x86_64:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu --manifest-path implants/mastodon/Cargo.toml
	cp implants/mastodon/target/x86_64-unknown-linux-gnu/release/$(prog) ./$(prog)_mastodon_x86_64
	@echo -e "[+] You can find \033[1;32m$(prog)_mastodon_x86_64\033[0m in your current folder."

mastodon_linux: check_rustup install_linux_deps replace_key mastodon_build_linux_x86_64 gen_key

mastodon_linux_x86_64: check_rustup install_linux_deps replace_key mastodon_build_linux_x86_64 gen_key


mastodon_build_macos:
	@export PATH="/usr/local/bin/osxcross/target/bin:$PATH"
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-apple-darwin --manifest-path implants/mastodon/Cargo.toml
	cp implants/mastodon/target/x86_64-apple-darwin/release/$(prog) ./$(prog)_mastodon_macOS
	@echo -e "[+] You can find \033[1;32m$(prog)_mastodon_macOS\033[0m in your current folder."

mastodon_macos: replace_key mastodon_build_macos install_macos_deps gen_key

mastodon_arm_musl: check_rustup install_cross
	cross build --target arm-unknown-linux-musleabi --release
	cp implants/mastodon/target/arm-unknown-linux-musleabi/release/$(prog) ./$(prog)_mastodon_arm_musl
	@echo -e "[+] You can find \033[1;32m$(prog)_mastodon_arm_musl\033[0m in your current folder."

mastodon_armv7: check_rustup install_cross
	cross build --target armv7-unknown-linux-gnueabihf --release
	cp implants/mastodon/target/armv7-unknown-linux-gnueabihf/release/$(prog) ./$(prog)_mastodon_armv7
	@echo -e "[+] You can find \033[1;32m$(prog)_mastodon_armv7\033[0m in your current folder."

# Makefile for Virustotal implant

virustotal_release: check_cargo
	cargo build --release --manifest-path implants/virustotal/Cargo.toml
	cp implants/virustotal/target/release/$(prog) ./$(prog)_virustotal_release
	@echo -e "[+] You can find \033[1;32m$(prog)_virustotal_release\033[0m release version in your current folder."

virustotal_debug: check_cargo
	cargo build --manifest-path implants/virustotal/Cargo.toml
	cp implants/virustotal/target/debug/$(prog) ./$(prog)_virustotal_debug
	@echo -e "[+] You can find \033[1;32m$(prog)_virustotal_debug\033[0m debug version in your current folder."

virustotal_doc: check_cargo
	cargo doc --open --no-deps --manifest-path implants/virustotal/Cargo.toml

virustotal_build_windows_x64:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-pc-windows-gnu --manifest-path  implants/virustotal/Cargo.toml
	cp implants/virustotal/target/x86_64-pc-windows-gnu/release/$(prog).exe ./$(prog)_virustotal_x64.exe
	@echo -e "[+] You can find \033[1;32m$(prog)_virustotal_x64.exe\033[0m in your current folder."

virustotal_build_windows_x86:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target i686-pc-windows-gnu --manifest-path implants/virustotal/Cargo.toml
	cp implants/virustotal/target/i686-pc-windows-gnu/release/$(prog).exe ./$(prog)_virustotal_x86.exe
	@echo -e "[+] You can find \033[1;32m$(prog)_virustotal_x86.exe\033[0m in your current folder."

virustotal_windows: check_rustup install_windows_deps replace_key virustotal_build_windows_x64 gen_key

virustotal_windows_x64: check_rustup install_windows_deps replace_key virustotal_build_windows_x64 gen_key

virustotal_windows_x86: check_rustup install_windows_deps replace_key virustotal_build_windows_x86 gen_key

virustotal_build_linux_aarch64:
	cross build --target aarch64-unknown-linux-gnu --release --manifest-path implants/virustotal/Cargo.toml
	cp implants/virustotal/target/aarch64-unknown-linux-gnu/release/$(prog) ./$(prog)_virustotal_aarch64
	@echo -e "[+] You can find \033[1;32m$(prog)_virustotal_aarch64\033[0m in your current folder."

virustotal_linux_aarch64: check_rustup install_cross replace_key virustotal_build_linux_aarch64  gen_key

virustotal_build_linux_x86_64:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu --manifest-path implants/virustotal/Cargo.toml
	cp implants/virustotal/target/x86_64-unknown-linux-gnu/release/$(prog) ./$(prog)_virustotal_x86_64
	@echo -e "[+] You can find \033[1;32m$(prog)_virustotal_x86_64\033[0m in your current folder."

virustotal_linux_x86_64: check_rustup install_linux_deps replace_key virustotal_build_linux_x86_64  gen_key

virustotal_linux: check_rustup install_linux_deps replace_key virustotal_build_linux_x86_64  gen_key

virustotal_build_macos:
	@export PATH="/usr/local/bin/osxcross/target/bin:$PATH"
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-apple-darwin --manifest-path implants/virustotal/Cargo.toml
	cp implants/virustotal/target/x86_64-apple-darwin/release/$(prog) ./$(prog)_virustotal_macOS
	@echo -e "[+] You can find \033[1;32m$(prog)_virustotal_macOS\033[0m in your current folder."

virustotal_macos: check_rustup install_cross install_macos_deps replace_key virustotal_build_macos gen_key

virustotal_arm_musl: check_rustup install_cross
	cross build --target arm-unknown-linux-musleabi --release
	cp implants/virustotal/target/arm-unknown-linux-musleabi/release/$(prog) ./$(prog)_virustotal_arm_musl
	@echo -e "[+] You can find \033[1;32m$(prog)_virustotal_arm_musl\033[0m in your current folder."

virustotal_armv7: check_rustup install_cross
	cross build --target armv7-unknown-linux-gnueabihf --release
	cp implants/virustotal/target/armv7-unknown-linux-gnueabihf/release/$(prog) ./$(prog)_virustotal_armv7
	@echo -e "[+] You can find \033[1;32m$(prog)_virustotal_armv7\033[0m in your current folder."

# Keys

export KEY:=$(shell echo `tr -dc A-Za-z0-9 < /dev/urandom | head -c 64`)
export LITCRYPT_ENCRYPT_KEY:=$(shell echo `tr -dc A-Za-z0-9 < /dev/urandom | head -c 64`)

gen_key:
	@echo -e "[+] AES key to use for C2 server: \033[1;32m$$KEY\033[0m"

replace_key: gen_key
	sed -i -E "s/let key = (lc\!\(\"[a-zA-Z0-9]{64})/let key = lc\!\(\"$$KEY/" implants/mastodon/src/main.rs
	sed -i -E "s/let key = (lc\!\(\"[a-zA-Z0-9]{64})/let key = lc\!\(\"$$KEY/" implants/virustotal/src/main.rs

# Makefile help

help:
	@echo ""
	@echo "REC2 Server:"
	@echo "usage: make c2server_debug"
	@echo "usage: make c2server_release"
	@echo "usage: make c2server_windows"
	@echo "usage: make c2server_windows_x64"
	@echo "usage: make c2server_windows_x86"
	@echo "usage: make c2server_linux"
	@echo "usage: make c2server_linux_aarch64"
	@echo "usage: make c2server_linux_x86_64"
	@echo "usage: make c2server_macos"
	@echo "usage: make c2server_arm_musl"
	@echo "usage: make c2server_armv7"
	@echo ""
	@echo "VirusTotal implant:"
	@echo "usage: make virustotal_debug"
	@echo "usage: make virustotal_release"
	@echo "usage: make virustotal_windows"
	@echo "usage: make virustotal_windows_x64"
	@echo "usage: make virustotal_windows_x86"
	@echo "usage: make virustotal_linux"
	@echo "usage: make virustotal_linux_aarch64"
	@echo "usage: make virustotal_linux_x86_64"
	@echo "usage: make virustotal_macos"
	@echo "usage: make virustotal_arm_musl"
	@echo "usage: make virustotal_armv7"
	@echo ""
	@echo "Mastodon implant:"
	@echo "usage: make mastodon_debug"
	@echo "usage: make mastodon_release"
	@echo "usage: make mastodon_windows"
	@echo "usage: make mastodon_windows_x64"
	@echo "usage: make mastodon_windows_x86"$
	@echo "usage: make mastodon_linux"
	@echo "usage: make mastodon_linux_aarch64"
	@echo "usage: make mastodon_linux_x86_64"
	@echo "usage: make mastodon_macos"
	@echo "usage: make mastodon_arm_musl"
	@echo "usage: make mastodon_armv7"
	@echo ""
	@echo "Dependencies:"
	@echo "usage: make install_windows_deps"
	@echo "usage: make install_macos_deps"
	@echo ""
	@echo "Documentation:"
	@echo "usage: make c2server_doc"
	@echo "usage: make virustotal_doc"
	@echo "usage: make mastodon_doc"
	@echo ""
	@echo "Cleaning:"
	@echo "usage: make clean"
	@echo ""