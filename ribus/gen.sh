#!/bin/bash

# These ones are troublesome that bindgen cannot generate correctly.
CUSTOME="\
pub type gsize = usize;
pub const FALSE: gboolean = 0;
pub const TRUE: gboolean = !FALSE;"

IN=ribus/ibus_wrapper.h
OUT=ribus/src/ffi/ibus.rs
bindgen \
	"$IN" \
	--output "$OUT" \
	--raw-line "$CUSTOME" \
	--opaque-type '_IBus[A-DF-NP-Z].*|_G[A-KM-RT-Z].*|_IBusEngineDesc.*|_IBusObserved.*' \
	--allowlist-function 'ibus_(bus|component|engine|engine_desc|factory|text).*' \
	--allowlist-function 'ibus_main|ibus_quit|ibus_init' \
	--allowlist-function 'g_list_free' \
	--allowlist-type 'IBus.*' \
	--allowlist-var 'IBUS_KEY_([0-9a-zA-Z]{1}|Control.*|Tab|Return.*|Delete|Home|KP_.*|Back.*|Insert|Hyper.*|Shift.*|Caps.*|space|asciitilde)' \
	--blocklist-type 'gsize|_?IBus(Serializable|Keymap|Panel|HotkeyProfile|ExtensionEvent|XEvent|Registry|Unicode|Emoji|ProcessKeyEventData).*' \
	--blocklist-type '_?(GAsyncResult|GCancellable|GAsyncReadyCallback)' \
	--blocklist-function 'ibus_(.*async.*|bus_set_global_shortcut_keys)' \
	--no-prepend-enum-name \
	--rustified-enum 'IBusBusNameFlag' \
	--default-macro-constant-type=unsigned \
	--use-core \
	--merge-extern-blocks \
	--no-layout-tests \
	--no-doc-comments \
	-- \
	--std=c99 \
	$(pkg-config --cflags-only-I ibus-1.0)
