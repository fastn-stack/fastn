# CLAUDE Instructions for fastn Specification Dimensions

## Intelligent Dimension Selection for Specifications

When creating or updating .rendered files, **intelligently choose dimensions** that demonstrate the component properly **without wasting space**.

### **Width Selection Guidelines**

#### **Mobile/Narrow (40-50 chars)**
Use for testing **compact layouts** and **mobile-like constraints**:
- Text components: 40ch shows text wrapping/adaptation
- Buttons: 40ch forces compact button sizing  
- Forms: 40ch tests input field responsiveness
- Layout: 40ch demonstrates stacking behavior

#### **Standard/Desktop (70-90 chars)**  
Use for **typical terminal usage** and **comfortable viewing**:
- Most components: 80ch is standard terminal width
- Documentation examples: 80ch fits most terminal setups
- Complex layouts: 80ch shows normal desktop behavior

#### **Wide/Large (100-140 chars)**
Use for **wide terminal testing** and **generous spacing**:
- Wide layouts: 120ch shows generous spacing behavior
- Multiple columns: 120ch demonstrates side-by-side content
- Large components: 120ch tests maximum width adaptation

### **Height Selection Guidelines**

#### **Compact (5-15 lines)**
Use for **simple components** that don't need much vertical space:
- Text components: 8-12 lines (content + breathing room)
- Buttons: 6-10 lines (compact interactive elements)
- Simple layouts: 10-15 lines (basic arrangements)

#### **Standard (20-40 lines)**
Use for **moderate components** with some content:
- Form groups: 25-35 lines (multiple inputs + labels)
- Card layouts: 20-30 lines (title + content + actions)
- Medium lists: 25-40 lines (several items visible)

#### **Large (50+ lines)**
Use for **complex components** that benefit from space:
- Full forms: 50-80 lines (many fields + validation)
- Data tables: 60-100 lines (header + multiple rows)  
- Complex layouts: 80-120 lines (nested components)

### **Dimension Selection Strategy**

#### **For Each Component, Ask:**
1. **What's the minimum** width/height to show this component properly?
2. **What's the ideal** width/height for comfortable viewing?  
3. **What's the maximum** useful size before space is wasted?

#### **Pick 3 Meaningful Dimensions:**
- **Constraint test** - Narrow width/height showing adaptation
- **Optimal test** - Comfortable size showing normal usage  
- **Generous test** - Wide/tall size showing spacious layout

### **Examples of Good Dimension Choices**

#### **Text with Border Component:**
```
# 40x8    ← Compact: Shows text + border in minimal space
# 80x12   ← Standard: Comfortable reading with breathing room  
# 120x12  ← Wide: Shows how text adapts to wide container
```

#### **Button Component:**
```
# 30x6    ← Compact: Minimal functional button
# 60x8    ← Standard: Comfortable button with padding
# 100x8   ← Wide: Shows button in wide container
```

#### **Form Component:**
```
# 50x15   ← Compact: Form fields stacked efficiently  
# 80x25   ← Standard: Comfortable form with labels
# 120x25  ← Wide: Form with side-by-side elements
```

#### **Layout/Column Component:**
```
# 40x20   ← Narrow: Forces vertical stacking behavior
# 80x30   ← Standard: Normal column layout
# 120x30  ← Wide: Column with generous margins
```

### **Avoid These Dimensions**

#### **❌ Wasteful Choices:**
- **Text in 200×300** - Huge empty space around small text
- **Button in 150×100** - Massive window for tiny button
- **Simple layouts in 500×400** - Unnecessary canvas size

#### **❌ Too Constrained:**
- **Complex forms in 20×5** - Can't show proper layout
- **Multi-column in 25×8** - Forces poor responsive behavior
- **Large components in tiny windows** - Defeats purpose

### **Responsive Testing Focus**

#### **Width is Most Critical:**
- **Layout changes** - Components stack, wrap, adapt horizontally
- **Text behavior** - Wrapping, truncation, overflow handling
- **Spacing adaptation** - Margins, padding adjust to width

#### **Height for Comfort:**
- **Breathing room** - Enough space for comfortable viewing
- **Content fitting** - All content visible without scrolling
- **Visual balance** - Proportional appearance

### **Quick Decision Rules**

#### **For Simple Components (text, button, input):**
- **Width**: 30-40 (compact), 70-80 (standard), 100-120 (wide)
- **Height**: Content + 4-8 lines breathing room

#### **For Layout Components (column, row, grid):**  
- **Width**: 40 (mobile), 80 (desktop), 120 (wide desktop)
- **Height**: Enough to show 3-5 child elements comfortably

#### **For Complex Components (forms, cards, modals):**
- **Width**: 60-80 (functional), 100-140 (comfortable)
- **Height**: Content + 20-30% breathing room

## Implementation Notes

### **When Updating Specs:**
1. **Look at current dimensions** - Are they appropriate?
2. **Test visually** - Does it look good in terminal?
3. **Check responsiveness** - Do the 3 dimensions show meaningful differences?
4. **Optimize if needed** - Reduce wasted space, ensure content fits

### **Golden Rule:**
**Show the component properly with no wasted space** - every line and column should have a purpose for the specification demonstration.

The goal is **efficient, meaningful demonstrations** of responsive component behavior, not arbitrary dimensions.