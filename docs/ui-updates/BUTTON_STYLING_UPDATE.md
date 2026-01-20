# Button Styling Update - Warp Theme

## Summary
All button styles have been updated to use custom Warp Terminal-inspired styling with proper colors, shadows, and interactive states.

## Button Types Updated

### 1. **WarpPrimaryButton** (`primary_button_style()`, `button_style()`)
- **Active**: Bright electric blue (`#00D9FF`) background with dark text
- **Hover**: Brighter blue with enhanced glow shadow
- **Pressed**: Slightly darker blue
- **Disabled**: Muted gray
- **Used for**: Main actions, "Send" button, DApp launch

### 2. **WarpSecondaryButton** (`secondary_button_style()`)
- **Active**: Dark background with border
- **Hover**: Lighter background with blue border and subtle glow
- **Pressed**: Darker pressed state
- **Disabled**: Muted with faded border
- **Used for**: Secondary actions, "Simulate", "Dry Run", "Force", "Hardware"

### 3. **WarpDangerButton** (`delete_button_style()`, `danger_button_style()`, `close_button_style()`)
- **Active**: Subtle pink-red background with colored text and border
- **Hover**: Solid error color background with dark text
- **Pressed**: Darker pressed state
- **Disabled**: Very subtle with muted colors
- **Used for**: Delete account (×), destructive actions

### 4. **WarpAccentButton** (`network_add_button_style()`)
- **Active**: Subtle green background with success-colored text
- **Hover**: More visible green with glow
- **Pressed**: Even more visible on press
- **Disabled**: Very subtle
- **Used for**: Add network (+), Add token (+)

### 5. **WarpDisabledButton** (`disabled_button_style()`)
- **All states**: Muted gray, non-interactive
- **Used for**: Truly disabled buttons

## Visual Features

### Shadows & Glows
- Primary buttons have cyan glow shadows
- Hover states increase shadow intensity and offset
- Error/danger buttons have red glow on hover
- Success/accent buttons have green glow on hover

### Border Radius
- All buttons use 8px radius for modern rounded corners
- Consistent with Warp Terminal's design language

### Color Palette
- **PRIMARY**: `#00D9FF` - Electric blue
- **SUCCESS**: `#3FB950` - Green
- **ERROR**: `#FF6B9D` - Pink-red
- **BACKGROUND_PRIMARY**: `#0D1117` - Deep blue-black
- **BACKGROUND_SECONDARY**: `#161B22` - Slightly lighter
- **BACKGROUND_TERTIARY**: `#21262D` - Even lighter
- **BORDER_PRIMARY**: `#30363D` - Subtle borders
- **TEXT_PRIMARY**: `#E6EDF3` - High contrast text
- **TEXT_MUTED**: `#8B949E` - Subdued text

## Implementation Details

All button styles implement `iced::widget::button::StyleSheet` with:
- `active()` - Default state
- `hovered()` - Mouse hover state
- `pressed()` - Mouse pressed state
- `disabled()` - Non-interactive state

Each appearance includes:
- `background` - Button background color
- `text_color` - Text color
- `border` - Border styling (radius, width, color)
- `shadow` - Shadow effect
- `shadow_offset` - Shadow offset vector

## Benefits

✅ **Consistent Warp Theme** - All UI elements now match Warp Terminal's aesthetic
✅ **Better Visibility** - Proper contrast and colors ensure buttons are always visible
✅ **Modern UX** - Hover effects and shadows provide clear feedback
✅ **Accessible** - High contrast text on backgrounds
✅ **Professional** - Polished appearance with attention to detail

## Testing

Compile the project:
```bash
cargo check
cargo run --bin vaughan
```

All buttons should now be visible with proper Warp-themed styling!
