#!/bin/bash

# TODO: //
# This script should be run via curl:
# sh -c "$(curl -fsSL https://fpm.dev/install.sh)"
# or via wget
# sh -c "$(wget -qO- https://fastn.dev/install.sh)"

# The [ -t 1 ] check only works when the function is not called from
# a subshell (like in `$(...)` or `(...)`, so this hack redefines the
# function at the top level to always return false when stdout is not
# a tty.
if [ -t 1 ]; then
  is_tty() {
    true
  }
else
  is_tty() {
    false
  }
fi

setup_colors() {
    if ! is_tty; then
        FMT_RED=""
        FMT_GREEN=""
        FMT_YELLOW=""
        FMT_BLUE=""
        FMT_BOLD=""
        FMT_ORANGE=""
        FMT_RESET=""
    else
        FMT_RED=$(printf '\033[31m')
        FMT_GREEN=$(printf '\033[32m')
        FMT_YELLOW=$(printf '\033[33m')
        FMT_BLUE=$(printf '\033[34m')
        FMT_BOLD=$(printf '\033[1m')
        FMT_ORANGE=$(printf '\033[38;5;208m')
        FMT_RESET=$(printf '\033[0m')
    fi
}

print_fastn_logo() {
    echo "${FMT_ORANGE}       .:~!!~^.                                                                                     "
    echo "     7B@@@@@@@&.                                                                                    "
    echo "   .J@@@@@@&&B?                                                .7GBBBP.                             "
    echo "   !#@@@@&^                                                    ^P@@@@@^                             "
    echo "..:5&@@@@#~.:.        .:~!!~^..             ..^~!!~^..       .:?B@@@@@?^:..   .::::.   .:~!~^:.     "
    echo "5#@@@@@@@@@@@#7   .!G&@@@@@@@@@#5^       :JB@@@@@@@@@@#Y^   :P@@@@@@@@@@@&J   G@@@@G77P@@@@@@@&5^   "
    echo "?5&@@@@@@@&&&5^  ~B@@@@@#GGB&@@@@@P.    ?&@@@@@BPPB&@@@@&Y. .?&@@@@@@@&&&G!   P@@@@&&@@@@@@@@@@@&7. "
    echo "   J&@@@@#:     .YB&&#P^    ^P@@@@@!   .&@@@@#7    :JB&##G:    ~G@@@@&~.      P@@@@@@Y^...~P@@@@@&^ "
    echo "   7&@@@@B.              ...!G@@@@@?.  .#@@@@@B7^..            ^5@@@@&^       P@@@@&5      :B@@@@@! "
    echo "   ?&@@@@B.       .!JPB#&@@@@@@@@@@?.   ^5@@@@@@@@@&#GY7:      ^P@@@@&^       P@@@@#?      :5&@@@@! "
    echo "   ?&@@@@B.     :P&@@@@@&BG5YG@@@@@?.     :!YG#&@@@@@@@@@G:    ^P@@@@&^       P@@@@#?      :5&@@@@! "
    echo "   ?&@@@@B.    .Y@@@@&?:    .!&@@@@?.           ..^J#@@@@@Y.   ^P@@@@&^       P@@@@#?      :5&@@@@! "
    echo "   ?&@@@@B.    :P@@@@B:     ?#@@@@@?.  ~#@&@&Y^    :Y&@@@@Y.   :5@@@@@J:.     P@@@@#?      :5&@@@@! "
    echo "   ?&@@@@B.     7@@@@@&B55G&&@@@@@@?.  .P@@@@@@BGPB&@@@@@B^     !&@@@@@@&P~   P@@@@#?      :5&@@@@! "
    echo "   ?&@@@@#.      ~P&@@@@@@@B7!@@@@@J.    :Y#@@@@@@@@@@#5~        ~P&@@@@@&5   G@@@@&J      :P@@@@@! "
    echo "    ......         .:^~~^:.   .....         .:^~!!~^:.             .:^~~^..   ......        ......  ${FMT_RESET}"
}

update_path() {
    local shell_config_file
    if [ -n "$ZSH_VERSION" ]; then
        shell_config_file="${HOME}/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        shell_config_file="${HOME}/.bashrc"
    else
        shell_config_file="${HOME}/.profile"
    fi

    # Check if the path is already added to the shell config file
    if ! grep -qF "export PATH=\"\$PATH:${DESTINATION_PATH}\"" "$shell_config_file"; then
        echo "export PATH=\"\$PATH:${DESTINATION_PATH}\"" >> "$shell_config_file"
        echo "Updated the PATH variable in $shell_config_file"
        echo "Please restart your terminal session to start using fastn."
    fi
}

command_exists() {
  command -v "$@" >/dev/null 2>&1
}

setup() {
    print_fastn_logo

    # Parse arguments
    while [ $# -gt 0 ]; do
        case $1 in
            --pre-release) PRE_RELEASE=true ;;
            --controller) CONTROLLER=true;;
        esac
    shift
    done

    if [[ $PRE_RELEASE ]]; then
        URL="https://api.github.com/repos/fastn-stack/fastn/releases"
        echo "Downloading the latest pre-release binaries"
    else
        URL="https://api.github.com/repos/fastn-stack/fastn/releases/latest"
        echo "Downloading the latest production ready binaries"
    fi

    DESTINATION_PATH="/usr/local/bin"

    if [ -d "$DESTINATION_PATH" ]; then
        DESTINATION_PATH=$DESTINATION_PATH
    else
        DESTINATION_PATH="${HOME}/.fastn/bin"
        mkdir -p $DESTINATION_PATH
    fi

    if [[ $CONTROLLER ]]; then 
        curl -# -L "$URL" | grep ".*\/releases\/download\/.*\/fastn_controller_linux.*" | head -2 | cut -d : -f 2,3 | tee /dev/tty | xargs -I % curl -# -O -J -L % > /dev/null
        mv fastn_controller_linux_musl_x86_64 "${DESTINATION_PATH}/fastn"
        mv fastn_controller_linux_musl_x86_64.d "${DESTINATION_PATH}/fastn.d"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        curl -# -L "$URL" | grep ".*\/releases\/download\/.*\/fastn_macos.*" | head -1 | cut -d : -f 2,3 | tee /dev/tty | xargs -I % curl -# -O -J -L % > /dev/null
        mv fastn_macos_x86_64 "${DESTINATION_PATH}/fastn"
    else
        curl -# -L "$URL" | grep ".*\/releases\/download\/.*\/fastn_linux.*" | head -2 | cut -d : -f 2,3 | tee /dev/tty | xargs -I % curl -# -O -J -L % > /dev/null
        mv fastn_linux_musl_x86_64 "${DESTINATION_PATH}/fastn"
        mv fastn_linux_musl_x86_64.d "${DESTINATION_PATH}/fastn.d"
    fi


    echo ""

    chmod +x "${DESTINATION_PATH}/fastn"*

    # Add fastn to PATH if not already done
    update_path

    echo "${FMT_GREEN}╭────────────────────────────────────────╮"
    echo "│                                        │"
    echo "│   fastn installation completed         │"
    echo "│                                        │"
    echo "│   Restart your terminal to apply       │"
    echo "│   the changes.                         │"
    echo "│                                        │"
    echo "│   Get started with fastn at:           │"
    echo "│   ${FMT_BLUE}https://fastn.com${FMT_RESET}${FMT_GREEN}                    │"
    echo "│                                        │"
    echo "╰────────────────────────────────────────╯${FMT_RESET}"
}

main() {
    setup_colors

    if ! command_exists curl; then
        echo "${FMT_RED}curl not found. Please install curl and execute the script once again${FMT_RESET}"
        exit 1
    fi
    setup "$@"
}

main "$@"
