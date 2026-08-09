#!/usr/bin/env sh
set -eu

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# GitHub repository
REPO="KarnesTH/nf-installer"
BINARY_NAME="nf-installer"

# Detect OS and architecture
detect_platform() {
    os=""
    arch=""

    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        *)          printf "${RED}Error: Unsupported operating system${NC}\n" >&2; exit 1 ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="aarch64" ;;
        *)              printf "${RED}Error: Unsupported architecture${NC}\n" >&2; exit 1 ;;
    esac

    printf "%s-%s" "$os" "$arch"
}

# Get latest release tag
get_latest_tag() {
    tag=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$tag" ]; then
        printf "${RED}Error: Failed to get latest version${NC}\n" >&2
        exit 1
    fi
    printf "%s" "$tag"
}

# Extract version from tag (remove 'v' prefix)
get_version_from_tag() {
    printf "%s" "${1#v}"
}

# Warn if fontconfig is missing, since nf-installer calls fc-cache
check_fontconfig() {
    if ! command -v fc-cache >/dev/null 2>&1; then
        printf "${YELLOW}Warning: fc-cache not found${NC}\n"
        printf "${YELLOW}Install fontconfig, otherwise fonts will not be registered${NC}\n"
    fi
}

# Download and install
main() {
    platform=$(detect_platform)
    tag=$(get_latest_tag)
    version=$(get_version_from_tag "$tag")

    printf "${GREEN}Installing nf-installer ${tag} for ${platform}...${NC}\n"

    download_url="https://github.com/${REPO}/releases/download/${tag}/${BINARY_NAME}_${version}-${platform}"

    install_dir="${HOME}/.local/bin"
    mkdir -p "$install_dir"

    printf "${YELLOW}Downloading from ${download_url}...${NC}\n"
    if ! curl -fsSL "$download_url" -o "${install_dir}/${BINARY_NAME}"; then
        printf "${RED}Error: Failed to download binary${NC}\n" >&2
        exit 1
    fi

    chmod +x "${install_dir}/${BINARY_NAME}"

    case ":${PATH}:" in
        *":${install_dir}:"*) ;;
        *)
            printf "${YELLOW}Warning: ${install_dir} is not in your PATH${NC}\n"
            printf "${YELLOW}Add this line to your shell profile (.bashrc, .zshrc, etc.):${NC}\n"
            printf "${GREEN}export PATH=\"\${HOME}/.local/bin:\${PATH}\"${NC}\n"
            ;;
    esac

    check_fontconfig

    printf "${GREEN}Successfully installed nf-installer to ${install_dir}/${BINARY_NAME}${NC}\n"
    printf "${GREEN}Run 'nf-installer --help' to get started${NC}\n"
}

main "$@"