# set dotenv-filename := "install.env"

author := 'Lzu Tao'
version := '0.1'
sudo := 'false'
real_sudo := if sudo == 'sudo' { 'sudo' } else { '' }

profile := "dev"
dest_dir := ''
prefix :=  dest_dir / "usr"
libexec := prefix / "libexec"
datadir := prefix / "share"
ibus_componentdir := datadir / "ibus/component"
cargo_target_dir := `echo ${CARGO_TARGET_DIR:-target}`
bindir := if profile == 'dev' {
  cargo_target_dir / 'debug'
} else {
  cargo_target_dir / profile
}
export DATADIR := datadir

bin_name := "ibus-engine-gokien"
xml_name := "gokien.xml"
replace := "s!@AUTHOR@!" + author + "!;" \
         + "s!@VERSION@!" + version + "!;" \
         + "s!@LIBEXECDIR@!" + libexec + "!;" \
         + "s!@PKGDATADIR@!" + datadir + "!;"
alias c := check
alias b := build
alias r := run

check:
  cargo check --package ibus-gokien

build:
  cargo build --profile={{profile}} --package ibus-gokien

# use `just run debug` for debugging purpose
run $RUST_LOG='info':
  #!/bin/bash
  set -eux
  export RUST_BACKTRACE=1
  cargo run --profile={{profile}} --package ibus-gokien

[confirm]
xml:
  sed "{{replace}}" {{xml_name}}.in > {{xml_name}}

ffi:
  bash ./ribus/gen.sh

install : build
  #!/bin/bash
  set -eux
  just xml || true
  {{real_sudo}} install --strip -t {{libexec}} {{bindir}}/{{bin_name}}
  {{real_sudo}} install -m 444 -t {{ibus_componentdir}} ./{{xml_name}}

uninstall :
  #!/bin/bash
  set -eux
  {{real_sudo}} rm {{libexec}}/{{bin_name}}
  {{real_sudo}} rm {{ibus_componentdir}}/{{xml_name}}
