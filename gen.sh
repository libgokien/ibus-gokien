#!/bin/sh
# install or clone header files of `libxcb-keysyms1-dev`

bindgen --allowlist-function 'xcb_key_symbols.*' \
	--allowlist-type 'xcb_key_symbols.*' \
	--merge-extern-blocks \
	--no-layout-tests \
	--no-derive-copy \
	--no-derive-debug \
	--output src/ffi/xcb.rs \
	/usr/include/xcb/xcb_keysyms.h

bindgen \
	--no-layout-tests \
	--no-derive-copy \
	--no-derive-debug \
	--output src/ffi/keysymdef.rs \
	/usr/include/X11/keysymdef.h \
	-- \
	-DXK_LATIN1 \
	-DXK_MISCELLANY
