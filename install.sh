#!/bin/bash

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Quindar Tone API - Setup Script${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to print status messages
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Check prerequisites
echo -e "${YELLOW}[1/4] Checking prerequisites...${NC}"
echo ""

MISSING_DEPS=0

# Check for Rust
if command_exists rustc && command_exists cargo; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    print_status "Rust found (version $RUST_VERSION)"
else
    print_error "Rust not found. Please install from https://rustup.rs/"
    MISSING_DEPS=1
fi

# Check for pkg-config (required for OpenSSL detection on Linux)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if command_exists pkg-config; then
        print_status "pkg-config found"
    else
        print_error "pkg-config not found"
        print_info "Install with: sudo apt install pkg-config (Debian/Ubuntu) or sudo yum install pkg-config (Fedora/RHEL)"
        MISSING_DEPS=1
    fi
fi

# Check for OpenSSL development libraries (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if pkg-config --exists openssl 2>/dev/null; then
        OPENSSL_VERSION=$(pkg-config --modversion openssl 2>/dev/null || echo "unknown")
        print_status "OpenSSL development libraries found (version $OPENSSL_VERSION)"
    else
        print_error "OpenSSL development libraries not found"
        print_info "Install with: sudo apt install libssl-dev (Debian/Ubuntu) or sudo yum install openssl-devel (Fedora/RHEL)"
        MISSING_DEPS=1
    fi
fi

# Check for system audio libraries (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if pkg-config --exists alsa 2>/dev/null; then
        print_status "ALSA libraries found"
    else
        print_warning "ALSA libraries not found. You may need to install libasound2-dev (Debian/Ubuntu) or alsa-lib-devel (Fedora/RHEL)"
        print_info "The build will attempt to continue, but may fail without audio libraries"
    fi
fi

echo ""

if [ $MISSING_DEPS -eq 1 ]; then
    print_error "Missing required dependencies. Please install them and run this script again."
    exit 1
fi

# Setup environment file
echo -e "${YELLOW}[2/4] Setting up environment configuration...${NC}"
echo ""

if [ -f ".env" ]; then
    print_warning ".env file already exists"
    read -p "Do you want to overwrite it? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Keeping existing .env file"
        SKIP_ENV=1
    else
        SKIP_ENV=0
    fi
else
    SKIP_ENV=0
fi

if [ $SKIP_ENV -eq 0 ]; then
    # Copy template
    cp .env.example .env
    print_status "Created .env from .env.example"

    # Interactive configuration
    echo ""
    echo "Choose your TTS provider:"
    echo "  1) Edge TTS (free, Microsoft Azure Neural Voices)"
    echo "  2) OpenAI TTS (premium, requires API key)"
    read -p "Enter choice [1]: " TTS_CHOICE
    TTS_CHOICE=${TTS_CHOICE:-1}

    if [ "$TTS_CHOICE" = "2" ]; then
        # OpenAI setup
        sed -i 's/DEFAULT_TTS=EDGE/DEFAULT_TTS=OPENAI/' .env
        echo ""
        read -p "Enter your OpenAI API key: " OPENAI_KEY
        if [ ! -z "$OPENAI_KEY" ]; then
            sed -i "s/OPENAI_API_KEY=your_openai_api_key_here/OPENAI_API_KEY=$OPENAI_KEY/" .env
            print_status "OpenAI TTS configured"
        else
            print_warning "No API key provided. You'll need to add it manually to .env"
        fi
    else
        print_status "Edge TTS configured (default)"
    fi

    # Tone configuration
    echo ""
    echo "Choose your default tone:"
    echo "  1) QUINDAR (classic NASA beep tones)"
    echo "  2) THREE-NOTE-CHIME (theater-style C-E-G chime)"
    echo "  3) NO-TONE (voice only, no tones)"
    read -p "Enter choice [1]: " TONE_CHOICE
    TONE_CHOICE=${TONE_CHOICE:-1}

    case $TONE_CHOICE in
        2)
            sed -i 's/DEFAULT_TONE=QUINDAR/DEFAULT_TONE=THREE-NOTE-CHIME/' .env
            print_status "Three-note chime configured"
            ;;
        3)
            sed -i 's/DEFAULT_TONE=QUINDAR/DEFAULT_TONE=NO-TONE/' .env
            print_status "No tone configured"
            ;;
        *)
            print_status "Quindar tone configured (default)"
            ;;
    esac
    echo ""
fi

# Build Rust project
echo -e "${YELLOW}[3/4] Building Rust project...${NC}"
echo ""
print_info "Running 'cargo build --release' (this may take a few minutes)..."
echo ""

if cargo build --release; then
    print_status "Build completed successfully"
else
    print_error "Build failed. Please check the error messages above."
    exit 1
fi

echo ""

# Verify installation
echo -e "${YELLOW}[4/4] Verifying installation...${NC}"
echo ""

if [ -f "target/release/quindar_api" ]; then
    print_status "Binary created: target/release/quindar_api"
elif [ -f "target/release/quindar_api.exe" ]; then
    print_status "Binary created: target/release/quindar_api.exe"
else
    print_warning "Binary not found in expected location"
fi

if [ -f ".env" ]; then
    print_status "Configuration file: .env"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Installation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo ""
echo "  1. Start the server:"
echo "     ${YELLOW}cargo run --release${NC}"
echo ""
echo "  2. Test the API (in another terminal):"
echo "     ${YELLOW}curl -X POST http://localhost:42069/play \\${NC}"
echo "       ${YELLOW}-H \"Content-Type: application/json\" \\${NC}"
echo "       ${YELLOW}-d '{\"text\": \"Hello from Quindar Tone API\"}'${NC}"
echo ""
echo "  3. Read the documentation:"
echo "     ${YELLOW}cat README.md${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo "  Edit .env to change TTS provider, voice, or API keys"
echo ""
print_status "Setup complete. Happy coding!"
