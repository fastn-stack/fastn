# fastn spec-viewer Help Screen Design

## Help Dialog Layout

### **Help Screen Content:**

```
┌─ fastn spec-viewer Help ─────────────────────────────────────────┐
│                                                                  │
│  📚 fastn Component Specification Browser                        │
│                                                                  │
│  🗂️  Navigation:                                                 │
│    ↑/↓       Navigate component list                             │
│    Enter     Select component (same as arrow selection)          │
│    PgUp/PgDn Scroll long previews (when content overflows)       │
│                                                                  │
│  🖥️  Preview Controls:                                            │
│    1         40-character preview width                          │
│    2         80-character preview width (default)                │
│    3         120-character preview width                         │
│    ←/→       Cycle between available widths                      │
│    R         Toggle responsive mode (follows terminal resize)    │
│                                                                  │
│  🎛️  View Controls:                                               │
│    F         Toggle fullscreen preview (hide tree + source)     │
│    T         Toggle file tree panel                              │
│    S         Toggle source panel                                 │
│    Tab       Cycle panel focus for keyboard scrolling           │
│                                                                  │
│  💾 File Operations:                                              │
│    Ctrl+S    Save current preview as .rendered file             │
│    Ctrl+R    Regenerate preview (refresh)                       │
│                                                                  │
│  ℹ️  Information:                                                 │
│    ?         Toggle this help dialog                             │
│    I         Show component info (properties, usage)            │
│    D         Toggle debug mode (show layout calculations)       │
│                                                                  │
│  🚪 Exit:                                                         │
│    Q         Quit application                                    │
│    Esc       Quit application                                    │
│    Ctrl+C    Force quit                                          │
│                                                                  │
│  💡 Tips:                                                         │
│    • Resize terminal in responsive mode to test layouts         │
│    • Use fullscreen mode for detailed component inspection      │
│    • Different widths help test responsive component behavior   │
│                                                                  │
│                                    Press ? or h to close help   │
└──────────────────────────────────────────────────────────────────┘
```

## Status Bar Design

### **Bottom Status Bar (Always Visible):**

```
┌─────────────────────────────── Status Bar ────────────────────────────────┐
│ text/with-border.ftd │ 80ch │ ↑/↓: Navigate │ 1/2/3: Width │ ?: Help │ Q: Quit │
└────────────────────────────────────────────────────────────────────────────┘
```

**Status Elements:**
- **Current file**: `text/with-border.ftd`
- **Current width**: `80ch` (40ch/80ch/120ch/Responsive)  
- **Quick shortcuts**: Most important actions
- **Help reminder**: `?` for full help

## Fullscreen Mode Help

### **Fullscreen Preview Help (Minimal):**

```
┌─ text/with-border.ftd @ 80ch ──────────────────────────── [F] Exit Fullscreen ┐
│                                                                                │
│  ┌─────────────────┐                                                          │
│  │                 │                                                          │
│  │  Hello World    │                                                          │
│  │                 │                                                          │
│  └─────────────────┘                                                          │
│                                                                                │
│                                                                                │
│                                                                                │
│  1/2/3: Width │ R: Responsive │ ?: Help │ Q: Quit                             │
└────────────────────────────────────────────────────────────────────────────────┘
```

## Component Information Dialog

### **Component Info (Triggered by 'I'):**

```
┌─ Component Information: text/with-border.ftd ────────────────────────────────┐
│                                                                              │
│  📝 Description:                                                             │
│    Text component with border and padding styling                           │
│                                                                              │
│  🏗️ Properties:                                                              │
│    • text: caption or body (required)                                       │
│    • border-width.px: integer (styling)                                     │
│    • padding.px: integer (spacing)                                          │  
│    • color: ftd.color (text color)                                          │
│                                                                              │
│  📐 Current Render:                                                          │
│    Width: 17 characters (text + padding + border)                           │
│    Height: 5 lines (text + padding + border)                                │
│    Layout: Single text element with box model                               │
│                                                                              │
│  🎯 Usage Examples:                                                          │
│    fastn spec-viewer text/with-border.ftd --stdout                          │
│    fastn spec-viewer text/with-border.ftd --stdout --width=120              │
│                                                                              │
│                                          Press I or Esc to close            │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Debug Mode Information

### **Debug Layout Info (Triggered by 'D'):**

```
┌─ Debug Information: text/with-border.ftd ────────────────────────────────────┐
│                                                                              │
│  📊 Layout Calculations:                                                     │
│    Taffy computed size: 88.0px × 16.0px                                     │
│    Character conversion: 11ch × 1ch                                         │
│    Content area: 11ch × 1ch                                                 │
│    Border area: +2ch × +2ch = 13ch × 3ch                                    │
│    Total rendered: 17ch × 5ch                                               │
│                                                                              │
│  🎨 Styling Applied:                                                         │
│    ✅ Border: Unicode box drawing (┌─┐│└┘)                                   │
│    ✅ Padding: 2ch horizontal, 1ch vertical                                  │
│    ✅ Color: ANSI red (\x1b[31m...\x1b[0m)                                   │
│    ✅ Text: "Hello World" (11 characters)                                    │
│                                                                              │
│  ⚙️ Rendering Pipeline:                                                       │
│    fastn source → Taffy layout → ASCII canvas → ANSI output                 │
│                                                                              │
│                                          Press D or Esc to close            │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Responsive Mode Indicator

### **Responsive Mode Status:**

```
┌─ Preview @ Responsive (127ch) ─ Terminal: 127×35 ─ [R] Fixed Mode ─────────┐
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  Hello World - adapts to your terminal width                       │   │  
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  💡 Resize terminal to test responsive behavior                             │
│     Current: 127ch × 35 lines                                              │
│                                                                             │
│  R: Fixed Width │ F: Fullscreen │ ?: Help │ Q: Quit                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Implementation Features Map

### **All Supported Interactions:**

#### **File Navigation:**
- `↑/↓` - Navigate component list with visual selection
- `Enter` - Confirm selection (redundant with arrow selection)
- `PgUp/PgDn` - Scroll preview content when it overflows panel

#### **Preview Controls:**
- `1/2/3` - Quick width switching (40/80/120 characters)
- `←/→` - Cycle between available widths sequentially  
- `R` - Toggle responsive mode (follow terminal resize)

#### **View Controls:**
- `F` - Fullscreen preview (hide tree and source panels)
- `T` - Toggle file tree panel visibility
- `S` - Toggle source panel visibility  
- `Tab` - Cycle focus between panels for scrolling

#### **Information & Debug:**
- `?` or `h` - Toggle help dialog overlay
- `I` - Component information dialog
- `D` - Debug layout calculations dialog

#### **File Operations:**
- `Ctrl+S` - Save current preview to .rendered file
- `Ctrl+R` - Force regenerate current preview

#### **Exit:**
- `Q` - Normal quit
- `Esc` - Cancel/quit (context sensitive)
- `Ctrl+C` - Force quit from anywhere

### **Progressive Disclosure:**
- **Beginners**: Status bar shows essential shortcuts
- **Intermediate**: Help dialog shows all features
- **Advanced**: Debug mode shows technical details

This comprehensive help system documents **every feature** and provides **multiple levels of guidance** for different user expertise levels.