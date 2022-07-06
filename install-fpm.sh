#!/bin/bash

# This script should be run via curl:
# sh -c "$(curl -fsSL https://fpm.dev/install.sh)"
# or via wget
# sh -c "$(wget -qO- https://fpm.dev/install.sh)"

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


command_exists() {
  command -v "$@" >/dev/null 2>&1
}

setup_colors() {
    if ! is_tty; then
        FMT_RED=""
        FMT_GREEN=""
        FMT_YELLOW=""
        FMT_BLUE=""
        FMT_BOLD=""
        FMT_RESET=""
    else
        FMT_RED=$(printf '\033[31m')
        FMT_GREEN=$(printf '\033[32m')
        FMT_YELLOW=$(printf '\033[33m')
        FMT_BLUE=$(printf '\033[34m')
        FMT_BOLD=$(printf '\033[1m')
        FMT_RESET=$(printf '\033[0m')
    fi
}



setup() {
    # Parse arguments
    while [ $# -gt 0 ]; do
        case $1 in
            --pre-release) PRE_RELEASE=true ;;
            --controller) CONTROLLER=true;;
        esac
    shift
    done

    if [[ $PRE_RELEASE ]]; then
        URL="https://api.github.com/repos/fifthtry/fpm/releases"
        echo "Downloading the latest pre-release binaries"
    else
        URL="https://api.github.com/repos/fifthtry/fpm/releases/latest"
        echo "Downloading the latest production ready binaries"
    fi

    DESTINATION_PATH="/usr/local/bin"

    if [ -d "$DESTINATION_PATH" ]; then
        DESTINATION_PATH=$DESTINATION_PATH
    else
        DESTINATION_PATH="${HOME}/.fpm/bin"
        mkdir -p $DESTINATION_PATH
    fi

    if [[ $CONTROLLER ]]; then 
        curl -s $URL | grep ".*\/releases\/download\/.*\/fpm_controller_linux.*" | head -2 | cut -d : -f 2,3 | tee /dev/tty | xargs -I % curl -O -J -L %
        mv fpm_controller_linux_musl_x86_64 "${DESTINATION_PATH}/fpm"
        mv fpm_controller_linux_musl_x86_64.d "${DESTINATION_PATH}/fpm.d"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        curl -s $URL | grep ".*\/releases\/download\/.*\/fpm_macos.*" | head -1 | cut -d : -f 2,3 | tee /dev/tty | xargs -I % curl -O -J -L %
        mv fpm_macos_x86_64 "${DESTINATION_PATH}/fpm"
    else
        curl -s $URL | grep ".*\/releases\/download\/.*\/fpm_linux.*" | head -2 | cut -d : -f 2,3 | tee /dev/tty | xargs -I % curl -O -J -L %
        mv fpm_linux_musl_x86_64 "${DESTINATION_PATH}/fpm"
        mv fpm_linux_musl_x86_64.d "${DESTINATION_PATH}/fpm.d"
    fi
    chmod +x "${DESTINATION_PATH}/fpm"*
    

    if ! [[ $DESTINATION_PATH == "/usr/local/bin" ]]; then 
        cat <<EOF
Unable to create a binary link for your system. Please add the follwing to your .bashrc/.zshrc file

${FMT_GREEN}PATH="\$PATH:${DESTINATION_PATH}"${FMT_RESET}

and reload the configuration/restart the terminal session
EOF
    fi
}

main() {
    setup_colors

    if ! command_exists curl; then
        echo "${FMT_RED}curl command not found. Please install the curl utility and execute the script once again${FMT_RESET}"
        exit 1
    fi
    setup "$@"
}

main "$@"
