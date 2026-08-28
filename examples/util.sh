set_keybind() {
    echo -ne "\x1ebind\x1f${1}\x1f${2}\x1f${3}"
}
set_show_binds() {
    echo -ne "\x1eshow_binds\x1ftrue"
}
add_header() {
    echo -ne "\x1eheader\x1f$1"
}
add_fzf_arg() {
    echo -ne "\x1efzf_arg\x1f$1"
}
set_preview() {
    echo -ne "\x1epreview\x1f$1"
}
options_end() {
    echo -ne "\x1d"
}
