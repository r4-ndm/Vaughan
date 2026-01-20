#!/bin/bash

echo "🔍 Vaughan DApp Platform GUI Diagnostics"
echo "=========================================="
echo ""

# Check GPU and driver info
echo "1. GPU Information:"
if command -v lspci &> /dev/null; then
    lspci | grep -E "VGA|3D" || echo "   No GPU info available"
fi
echo ""

# Check display server
echo "2. Display Server:"
echo "   Session Type: ${XDG_SESSION_TYPE:-unknown}"
echo "   Wayland Display: ${WAYLAND_DISPLAY:-not set}"
echo "   X11 Display: ${DISPLAY:-not set}"
echo ""

# Check compositor
echo "3. Window Manager/Compositor:"
if [ -n "$DESKTOP_SESSION" ]; then
    echo "   Desktop Session: $DESKTOP_SESSION"
fi
if command -v wmctrl &> /dev/null; then
    echo -n "   Window Manager: "
    wmctrl -m | grep "Name:" | cut -d: -f2
fi
echo ""

# Check OpenGL info
echo "4. OpenGL Information:"
if command -v glxinfo &> /dev/null; then
    echo -n "   OpenGL Vendor: "
    glxinfo 2>/dev/null | grep "OpenGL vendor" | cut -d: -f2 || echo "unknown"
    echo -n "   OpenGL Renderer: "
    glxinfo 2>/dev/null | grep "OpenGL renderer" | cut -d: -f2 || echo "unknown"
else
    echo "   glxinfo not available - install mesa-utils for more info"
fi
echo ""

echo "5. Potential Fixes for GUI Artifacts:"
echo "======================================"
echo ""
echo "Option A - Software Rendering (most compatible):"
echo "  ICED_BACKEND=tiny-skia cargo run --bin dapp-platform --release"
echo ""
echo "Option B - Disable compositor effects:"
echo "  For KDE: Alt+Shift+F12 to toggle compositor"
echo "  For GNOME: gsettings set org.gnome.desktop.interface enable-animations false"
echo "  For XFCE: xfwm4 --replace --compositor=off"
echo ""
echo "Option C - Force specific rendering:"
echo "  LIBGL_ALWAYS_SOFTWARE=1 cargo run --bin dapp-platform --release"
echo ""
echo "Option D - Adjust DPI scaling:"
echo "  GDK_SCALE=1 GDK_DPI_SCALE=1 cargo run --bin dapp-platform --release"
echo ""
echo "Option E - Clear GPU shader cache:"
echo "  rm -rf ~/.cache/mesa_shader_cache"
echo "  rm -rf ~/.cache/nvidia/GLCache" 
echo ""
echo "Option F - Try different Iced backend:"
echo "  # Edit Cargo.toml and change iced features"
echo "  # Remove 'canvas' and 'advanced' features temporarily"
echo ""

# Test which backend works best
echo "6. Quick Test - Trying different backends..."
echo "============================================"
echo ""

echo "Testing with software renderer..."
timeout 3 bash -c 'ICED_BACKEND=tiny-skia cargo run --bin dapp-platform --release 2>&1 | grep -q "error"'
if [ $? -eq 0 ]; then
    echo "  ❌ Software renderer has issues"
else
    echo "  ✅ Software renderer appears to work"
fi

echo ""
echo "Recommended command to run based on your system (X11):"
echo "======================================================="
echo ""
echo "./fix_gui_artifacts.sh"
echo ""
echo "Or manually:"
echo "ICED_BACKEND=tiny-skia GDK_BACKEND=x11 cargo run --bin dapp-platform --release"