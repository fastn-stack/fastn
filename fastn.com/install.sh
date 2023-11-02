#!/bin/sh

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
        FMT_RED="$(printf '\033[31m')"
        FMT_GREEN="$(printf '\033[32m')"
        FMT_YELLOW="$(printf '\033[33m')"
        FMT_BLUE="$(printf '\033[34m')"
        FMT_BOLD="$(printf '\033[1m')"
        FMT_ORANGE="$(printf '\033[38;5;208m')"
        FMT_RESET="$(printf '\033[0m')"
    fi
}

print_fastn_logo() {
    echo $FMT_ORANGE
    echo "      :--===--                                                                            "
    echo "    .++++++++=                                                                            "
    echo "    =++++=::-.                                             =++++:                         "
    echo "   .+++++.                                                 =++++:                         "
    echo ":--=+++++=---     .:--====--:.         .---=====-:.     ---+++++=---.  .-----  :-====-:.  "
    echo "-++++++++++++   .=++++++++++++=.     :=++++++++++++=:   ++++++++++++.  .+++++.=+++++++++= "
    echo "...:+++++:...  :+++++-...:=+++++.   .+++++-.  .:+++++.  ...+++++-...   .+++++++-::-=+++++-"
    echo "   .+++++       ....      .+++++-   :+++++:.     ....      =++++:      .+++++-      -++++="
    echo "   .+++++          .:---=+++++++-    -+++++++==--.         =++++:      .+++++.      :++++="
    echo "   .+++++      .-=+++++==--+++++-     .:-==++++++++=-.     =++++:      .+++++.      :++++="
    echo "   .+++++     .+++++:      =++++-            .:-++++++     =++++:      .+++++.      :++++="
    echo "   .+++++     .++++=      :+++++-   -----:      :+++++     =++++:      .+++++.      :++++="
    echo "   .+++++      +++++-:::-=+=++++-   .=++++=-::-=++++=:     -+++++==-   .+++++.      :++++="
    echo "   .+++++       -++++++++-.-++++-     .=++++++++++-:        =+++++++:  .+++++.      :++++="
    echo $FMT_RESET
}

print_success_box() {
    log_message "╭────────────────────────────────────────╮"
    log_message "│                                        │"
    log_message "│   fastn installation completed         │"
    log_message "│                                        │"
    log_message "│   Restart your terminal to apply       │"
    log_message "│   the changes.                         │"
    log_message "│                                        │"
    log_message "│   Get started with fastn at:           │"
    log_message "│   ${FMT_BLUE}https://fastn.com${FMT_RESET}                    │"
    log_message "│                                        │"
    log_message "╰────────────────────────────────────────╯"
}

# Function for logging informational messages
log_message() {
    echo "${FMT_GREEN}$1${FMT_RESET}"
}

# Function for logging error messages
log_error() {
    echo "${FMT_RED}ERROR:${FMT_RESET} $1"
}

command_exists() {
  command -v "$@" >/dev/null 2>&1
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

    echo ""
    
    # Create the shell config file if it doesn't exist
    if [ ! -e "$shell_config_file" ]; then
        touch "$shell_config_file"
    fi

    # Check if the path is already added to the shell config file
    if ! grep -qF "export PATH=\"\$PATH:${DESTINATION_PATH}\"" "$shell_config_file"; then
        if [ -w "$shell_config_file" ]; then
            # Add the destination path to the PATH variable in the shell config file
            echo "export PATH=\"\$PATH:${DESTINATION_PATH}\"" >> "$shell_config_file" &&
            return 0
        else
            log_error "Failed to add '${DESTINATION_PATH}' to PATH. Insufficient permissions for '$shell_config_file'."
            log_message "The installer has successfully downloaded the \`fastn\` binary in '${DESTINATION_PATH}' but it failed to add it in your \$PATH variable."
            log_message "Configure the \$PATH manually or run \`fastn\` binary from '${DESTINATION_PATH}/fastn'"
            return 1
        fi
    else
        return 0
    fi
}

remove_temp_files() {
    rm -f fastn_macos_x86_64 fastn_linux_musl_x86_64 fastn_controller_linux_musl_x86_64
}

# Function to handle Ctrl+C
exit_on_interrupt() {
    log_error "Installation interrupted."
    remove_temp_files
    exit 1
}

setup() {
    # Trap Ctrl+C and call the exit_on_interrupt function
    trap exit_on_interrupt INT

    PRE_RELEASE=""
    CONTROLLER=""
    VERSION=""

    # Parse arguments
    while [ $# -gt 0 ]; do
        case $1 in
            --pre-release) PRE_RELEASE=true ;;
            --controller) CONTROLLER=true ;;
            --version=*) VERSION="${1#*=}" ;;
            *) echo "Unknown CLI argument: $1"; exit 1 ;;
        esac
        shift
    done

    if [ -n "$VERSION" ]; then
        echo "Installing fastn version: $VERSION"
    fi

    if [ -n "$PRE_RELEASE" ]; then
        URL="https://github.com/fastn-stack/fastn/releases/latest/download"
        log_message "Downloading the latest pre-release binaries"
    elif [ -n "$VERSION" ]; then
        URL="https://github.com/fastn-stack/fastn/releases/download/$VERSION"
        log_message "Downloading fastn release $VERSION binaries"
    else
        URL="https://github.com/fastn-stack/fastn/releases/latest/download"
        log_message "Downloading the latest production-ready binaries"
    fi

    DESTINATION_PATH="/usr/local/bin"

    if [ -d "$DESTINATION_PATH" ] && [ -w "$DESTINATION_PATH" ]; then
        DESTINATION_PATH=$DESTINATION_PATH
    else
        DESTINATION_PATH="${HOME}/.fastn/bin"
        mkdir -p "$DESTINATION_PATH"
    fi

    # Remove temporary files from previous install attempts
    remove_temp_files

    if [ -n "$CONTROLLER" ]; then
        if [ "$(uname)" = "Darwin" ]; then
            FILENAME="fastn_controller_macos_x86_64"
        else
            FILENAME="fastn_controller_linux_musl_x86_64"
        fi
    else
        if [ "$(uname)" = "Darwin" ]; then
            FILENAME="fastn_macos_x86_64"
        else
            FILENAME="fastn_linux_musl_x86_64"
        fi
    fi

    # Download the binary directly using the URL
    curl -# -L -o "${DESTINATION_PATH}/fastn" "${URL}/${FILENAME}"
    chmod +x "${DESTINATION_PATH}/fastn"

    if [ -n "$CONTROLLER" ]; then
        if [ "$(uname)" = "Darwin" ]; then
            FILENAME="fastn_controller_macos_x86_64.d"
        else
            FILENAME="fastn_controller_linux_musl_x86_64.d"
        fi
        curl -# -L -o "${DESTINATION_PATH}/fastn.d" "${URL}/${FILENAME}"
    fi

    # Check if the destination files are moved successfully before setting permissions
    if [ -e "${DESTINATION_PATH}/fastn" ]; then
        if update_path; then
            print_success_box
        fi
    else
        log_error "Installation failed. Please check if you have sufficient permissions to install in $DESTINATION_PATH."
    fi

    # Remove temporary files from this install attempt
    remove_temp_files
}


main() {
    setup_colors
    print_fastn_logo

    if ! command_exists curl; then
        log_error "curl not found. Please install curl and execute the script once again"
        exit 1
    fi
    setup "$@"
}

main "$@"
