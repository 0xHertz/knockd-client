#!/bin/bash
set -e

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BIN_PATH="/usr/bin/knockd-client"

echo -e "${CYAN}=== Knockd Client Installer ===${NC}"

EXT_ID="$1"
BROWSER="${2:-chrome}"
if [ -z "$EXT_ID" ]; then
    echo "Usage: $0 <extension-id> [chrome|edge|chromium|brave]"
    echo "  Get extension ID from chrome://extensions after loading unpacked"
    exit 1
fi
echo -e "Extension: ${GREEN}$EXT_ID${NC}  Browser: ${GREEN}$BROWSER${NC}"

echo -e "${CYAN}[1/3] Checking binary...${NC}"
if [ ! -f "$BIN_PATH" ]; then
    echo -e "  ${RED}$BIN_PATH not found. Install .deb first.${NC}"
    exit 1
fi
echo -e "  ${GREEN}$BIN_PATH${NC}"

echo -e "${CYAN}[2/3] Registering Native Messaging Host...${NC}"
case "$BROWSER" in
    chrome)    NHDIR="$HOME/.config/google-chrome/NativeMessagingHosts" ;;
    edge)      NHDIR="$HOME/.config/microsoft-edge/NativeMessagingHosts" ;;
    chromium)  NHDIR="$HOME/.config/chromium/NativeMessagingHosts" ;;
    brave)     NHDIR="$HOME/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts" ;;
    *) echo -e "${RED}Unknown browser: $BROWSER${NC}"; exit 1 ;;
esac
mkdir -p "$NHDIR"
cat > "$NHDIR/com.knockd.client.json" << JSONEOF
{
  "name": "com.knockd.client",
  "description": "Knockd Client - SPA auth gateway",
  "path": "$BIN_PATH",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://${EXT_ID}/"]
}
JSONEOF
echo -e "  ${GREEN}$NHDIR/com.knockd.client.json${NC}"

echo -e "${CYAN}[3/3] Load Extension${NC}"
echo "  chrome://extensions → Developer mode → Load unpacked"
echo "  Select: ${GREEN}$PROJECT_DIR/extension/${NC}"
echo ""
echo -e "${GREEN}=== Done ===${NC}"
echo "Restart Chrome for changes to take effect."
