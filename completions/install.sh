#!/bin/bash
# Copyright (c) 2026 Analog Devices, Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# Installation script for Code in Motion (cim) bash completions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPLETION_FILE="$SCRIPT_DIR/cim.bash"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Install bash completions for cim"
    echo ""
    echo "Options:"
    echo "  --system      Install system-wide (requires sudo)"
    echo "  --user        Install for current user only"
    echo "  --source      Add source line to ~/.bashrc"
    echo "  --help        Show this help message"
    echo ""
    echo "If no option is specified, the script will try to detect the best method."
}

install_system_wide() {
    echo -e "${BLUE}Installing system-wide bash completion...${NC}"
    
    # Detect the system and appropriate directory
    if [[ -d "/usr/share/bash-completion/completions" ]]; then
        # Ubuntu/Debian style
        DEST_DIR="/usr/share/bash-completion/completions"
        DEST_FILE="$DEST_DIR/cim"
    elif [[ -d "/etc/bash_completion.d" ]]; then
        # CentOS/RHEL/Fedora style
        DEST_DIR="/etc/bash_completion.d"
        DEST_FILE="$DEST_DIR/cim"
    elif command -v brew >/dev/null 2>&1 && [[ -d "$(brew --prefix)/etc/bash_completion.d" ]]; then
        # macOS with Homebrew
        DEST_DIR="$(brew --prefix)/etc/bash_completion.d"
        DEST_FILE="$DEST_DIR/cim"
    else
        echo -e "${RED}Error: Could not find system bash completion directory${NC}"
        echo "Please install bash-completion package or use --user option"
        exit 1
    fi
    
    if [[ ! -w "$DEST_DIR" ]]; then
        echo -e "${YELLOW}Need sudo access to write to $DEST_DIR${NC}"
        sudo cp "$COMPLETION_FILE" "$DEST_FILE"
    else
        cp "$COMPLETION_FILE" "$DEST_FILE"
    fi
    
    echo -e "${GREEN}✓ Installed completion to $DEST_FILE${NC}"
    echo -e "${YELLOW}Note: You may need to start a new terminal session for completions to work${NC}"
}

install_user() {
    echo -e "${BLUE}Installing user-specific bash completion...${NC}"
    
    # Create user completion directory
    USER_COMPLETION_DIR="$HOME/.local/share/bash-completion/completions"
    mkdir -p "$USER_COMPLETION_DIR"
    
    # Copy completion file
    DEST_FILE="$USER_COMPLETION_DIR/cim"
    cp "$COMPLETION_FILE" "$DEST_FILE"
    
    echo -e "${GREEN}✓ Installed completion to $DEST_FILE${NC}"
    
    # Check if it's already sourced in bashrc
    if ! grep -q "bash-completion/completions/cim" "$HOME/.bashrc" 2>/dev/null; then
        echo -e "${YELLOW}Adding source line to ~/.bashrc...${NC}"
        echo "" >> "$HOME/.bashrc"
        echo "# Code in Motion (cim) bash completion" >> "$HOME/.bashrc"
        echo "source ~/.local/share/bash-completion/completions/cim" >> "$HOME/.bashrc"
        echo -e "${GREEN}✓ Added source line to ~/.bashrc${NC}"
    else
        echo -e "${YELLOW}Source line already exists in ~/.bashrc${NC}"
    fi
    
    echo -e "${YELLOW}Run 'source ~/.bashrc' or start a new terminal to enable completions${NC}"
}

install_source() {
    echo -e "${BLUE}Adding direct source to ~/.bashrc...${NC}"
    
    COMPLETION_PATH="$(cd "$SCRIPT_DIR" && pwd)/cim.bash"
    
    if ! grep -q "$COMPLETION_PATH" "$HOME/.bashrc" 2>/dev/null; then
        echo "" >> "$HOME/.bashrc"
        echo "# Code in Motion (cim) bash completion" >> "$HOME/.bashrc"
        echo "source '$COMPLETION_PATH'" >> "$HOME/.bashrc"
        echo -e "${GREEN}✓ Added source line to ~/.bashrc${NC}"
    else
        echo -e "${YELLOW}Source line already exists in ~/.bashrc${NC}"
    fi
    
    echo -e "${YELLOW}Run 'source ~/.bashrc' or start a new terminal to enable completions${NC}"
}

check_dependencies() {
    # Check if bash-completion is available
    if ! command -v complete >/dev/null 2>&1; then
        echo -e "${RED}Error: Bash completion is not available${NC}"
        echo "Please install the bash-completion package:"
        echo ""
        if command -v apt-get >/dev/null 2>&1; then
            echo "  sudo apt-get install bash-completion"
        elif command -v yum >/dev/null 2>&1; then
            echo "  sudo yum install bash-completion"
        elif command -v dnf >/dev/null 2>&1; then
            echo "  sudo dnf install bash-completion"
        elif command -v brew >/dev/null 2>&1; then
            echo "  brew install bash-completion"
        fi
        exit 1
    fi
}

auto_detect_method() {
    echo -e "${BLUE}Auto-detecting installation method...${NC}"
    
    # Try system-wide first if we have permissions or can use sudo
    if [[ -d "/usr/share/bash-completion/completions" && -w "/usr/share/bash-completion/completions" ]]; then
        echo -e "${YELLOW}Using system-wide installation (writable directory found)${NC}"
        install_system_wide
    elif [[ -d "/etc/bash_completion.d" && -w "/etc/bash_completion.d" ]]; then
        echo -e "${YELLOW}Using system-wide installation (writable directory found)${NC}"
        install_system_wide
    elif command -v sudo >/dev/null 2>&1; then
        echo -e "${YELLOW}System directory requires sudo, using system-wide installation${NC}"
        install_system_wide
    else
        echo -e "${YELLOW}No sudo access, using user installation${NC}"
        install_user
    fi
}

main() {
    # Check if completion file exists
    if [[ ! -f "$COMPLETION_FILE" ]]; then
        echo -e "${RED}Error: Completion file not found: $COMPLETION_FILE${NC}"
        echo "Please run this script from the completions directory"
        exit 1
    fi
    
    check_dependencies
    
    case "${1:-}" in
        --system)
            install_system_wide
            ;;
        --user)
            install_user
            ;;
        --source)
            install_source
            ;;
        --help|-h)
            print_usage
            exit 0
            ;;
        "")
            auto_detect_method
            ;;
        *)
            echo -e "${RED}Error: Unknown option: $1${NC}"
            print_usage
            exit 1
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}Installation complete!${NC}"
    echo -e "Test the completion by typing: ${BLUE}cim <TAB><TAB>${NC}"
}

main "$@"