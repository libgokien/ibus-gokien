# ibus-gokien - Vietnamese input method engine for IBus

Currently only support TELEX. Please open an issue/PR if you're interested
at other input methods like VNI.

## Install and test with log

```bash
ibus read-config
env DCONF_PROFILE=ibus dconf read /desktop/ibus/general/preload-engines
ibus-setup
gsettings get org.gnome.desktop.input-sources sources
```

Standalone:
```
just run debug
ibus engine gokien
```

## Known bugs

N/A.

### Unsupported/Hard features

* surrounding_text: sometimes work in firefox textbox

* emulate fake backspace: `ibus_engine_forward_key_event`
  won't implement. IBus is too buggy and warn people about this.

  https://github.com/BoGoEngine/ibus-bogo/issues/216

* Fcitx https://github.com/fcitx/fcitx5-unikey
  + too MUCH c++. no c api. Rust can use cxx crate but no thanks.
  + https://fcitx-im.org/wiki/Develop_an_simple_input_method
  + https://codedocs.xyz/fcitx/fcitx5/group__FcitxCore.html
  + apt download fcitx-core-dev
  + can view source of fcitx5-bamboo and fcitx-unikey for API usage
