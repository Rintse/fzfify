# Dependencies: pactl, jq

set_keybind() {
    echo -ne "\x1ebind\x1f${1}\x1f${2}\x1f${3}"
}

list_items() {
    default=$(pactl "get-default-$1")
    pactl -f json list "${1}s" |
        jq "sort_by(.name != \"$default\")" |
        jq -r '.[] | "\(.index) | \(.description)"' |
        column -s"|" -o" " -t
}

preview_item() {
    pactl -f json list "${1}s" |
        jq --arg idx "$2" '.[] | select(.index == ($idx | tonumber))' |
        jq '{name, description, driver, sample_specification, mute, volume, balance}'
}

show_details() {
    pactl -f json list "${1}s" |
        jq --arg idx "$2" '.[] | select(.index == ($idx | tonumber))' |
        less
}

change_volume() {
    pactl "set-$1-volume" "$2" "$3"
}

toggle_mute() {
    pactl "set-$1-mute" "$2" toggle
}

set_default() {
    pactl "set-default-$1" "$2"
}

show_list() {
    [[ "$1" == "source" ]] && other="sink" || other="source"

    echo -ne "\x1eshow_binds\x1ftrue"
    echo -ne "\x1epreview\x1f{{*}} preview $1 {1}"
    set_keybind "Volume down" \
        "ctrl-j" \
        "execute-silent({{*}} vol $1 {1} -5%)+refresh-preview"
    set_keybind "Volume up" \
        "ctrl-k" \
        "execute-silent({{*}} vol $1 {1} +5%)+refresh-preview"
    set_keybind "Mute" \
        "ctrl-m" \
        "execute-silent({{*}} mute $1 {1})+refresh-preview"
    set_keybind "Set default" \
        "ctrl-d" \
        "execute-silent({{*}} setdef $1 {1})+become({{}})"
    set_keybind "Show full details" \
        "ctrl-o" \
        "execute({{*}} details $1 {1})"
    set_keybind "Switch to ${other}s" \
        "ctrl-s" \
        "become({{0}} ${other}s)"
    echo -ne "\x1d"

    list_items "$1"
}

case "$1" in
"" | sinks)
    show_list sink
    ;;
sources)
    show_list source
    ;;
preview)
    preview_item "$2" "$3"
    ;;
vol)
    change_volume "$2" "$3" "$4"
    ;;
mute)
    toggle_mute "$2" "$3"
    ;;
setdef)
    set_default "$2" "$3"
    ;;
details)
    show_details "$2" "$3"
    ;;
*)
    echo "wat"
    ;;
esac
