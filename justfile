# set dotenv-filename := "install.env"

prefix := "/usr"
libexec := prefix / "libexec"
datadir := prefix / "share"
ibus_componentdir := datadir / "ibus/component"
author := 'Lzu Tao'
version := '0.1'

export DATADIR := datadir

profile := "debug"
bin_name := "ibus-engine-gokien"
xml_name := "gokien.xml"
replace := "\
s!@AUTHOR@!" + author + "!;\
s!@VERSION@!" + version + "!;\
s!@LIBEXECDIR@!" + libexec + "!;\
s!@PKGDATADIR@!" + datadir + "!;\
s!@PACKAGE_BUGREPORT@!to be defined!;"

check:
  cargo check

build:
  #!/bin/bash
  set -eux
  if [[ {{profile}} == "release" ]]; then
    cargo build --release
  else
    cargo build
  fi

run: xml
  #!/bin/bash
  set -eux
  if [[ {{profile}} == "release" ]]; then
    cargo run --release
  else
    RUST_LOG=debug cargo run
  fi

[confirm]
xml:
  sed "{{replace}}" {{xml_name}}.in > {{xml_name}}

[confirm]
ffi:
  bash ./ribus/gen.sh

install sudo="false": (build)
  #!/bin/bash
  set -eux
  just xml || true
  CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-target/}
  if [[ {{sudo}} == "sudo" ]]; then
    sudo=sudo
  else
    sudo=
  fi
  ${sudo} install -t {{libexec}} "${CARGO_TARGET_DIR}"/{{profile}}/{{bin_name}}
  ${sudo} install -m 444 -t {{ibus_componentdir}} ./{{xml_name}}

uninstall sudo='false':
  #!/bin/bash
  set -eux
  if [[ {{sudo}} == "sudo" ]]; then
    sudo=sudo
  else
    sudo=
  fi
  ${sudo} rm {{libexec}}/{{bin_name}}
  ${sudo} rm {{ibus_componentdir}}/{{xml_name}}
