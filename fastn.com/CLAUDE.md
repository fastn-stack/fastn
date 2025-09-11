# Claude Instructions for fastn.com

## Testing FTD Files

**IMPORTANT**: Before committing any changes to `.ftd` files in this repository:

1. **Use RELEASE mode fastn only**:
   - **ALWAYS** build fastn from source:
     ```bash
     cd /Users/amitu/Projects/fastn
     cargo build --release --bin fastn
     cp target/release/fastn ~/.cargo/bin/fastn
     ```
   - Only use `~/.cargo/bin/fastn` (release build from local source)
   - **NEVER** use debug builds as they are unstable and will crash
   - **NEVER** use `cargo install fastn` (gets wrong package from crates.io)
   - The installation script binary may have dependency issues - use local source

2. **Start the fastn development server**:
   ```bash
   cd fastn.com
   ~/.cargo/bin/fastn serve --port 8001
   ```

2. **Test your changes in browser**:
   - Open the modified pages in browser via `http://127.0.0.1:8001/path/to/your/file`
   - Ensure the page loads without compilation errors
   - Check that all content renders correctly
   - Verify any components or imports work as expected

3. **Common paths to test**:
   - Language spec: `http://127.0.0.1:8001/language-spec/`
   - FTD components: `http://127.0.0.1:8001/ftd/audio/` 
   - New documentation: `http://127.0.0.1:8001/your-new-path/`

4. **Fix any compilation errors**:
   - Check the fastn server output for error messages
   - Common issues: missing imports, incorrect syntax, circular imports
   - Test changes iteratively until they work

5. **Only commit after successful testing**

## Common FTD Patterns

- Use existing imports like `fastn.com/assets` and design system components
- Check existing files for proper component structure and naming
- Follow the established documentation patterns in `/ftd/` directory

## Lint/TypeCheck Commands

None specifically required for FTD files - the fastn server will catch syntax errors.

## FTD Common Mistakes Cheat Sheet

### ❌ Most Common Errors and Solutions

**1. Mismatched Section Ends**
```
Error: "No section found to end: component-name"
```
**Cause**: Unmatched `-- end:` statements in code blocks or missing escape
**Fix**: Escape end statements in code blocks: `\-- end: component-name`

**2. Wrong Type References**
```
Error: "fastn.type.heading-large not found"
```
**Cause**: Using `$fastn.type.*` instead of `$inherited.types.*`
**Fix**: Use `$inherited.types.heading-large` not `$fastn.type.heading-large`

**3. Component Attribute Not Found**
```
Error: "Header type 'preload' not found for component 'ftd.audio'"
```
**Cause**: Using unsupported attributes for components
**Fix**: Check existing component docs, only use supported attributes

**4. Unescaped Code in Examples**
```
Error: Section parsing errors in code blocks
```
**Cause**: FTD code in `-- ds.code:` blocks needs escaping
**Fix**: Escape FTD syntax: `\-- ftd.text:` not `-- ftd.text:`

**5. Custom Components Don't Exist**
```
Error: "terminal-output not found"
```  
**Cause**: Using imaginary components like `ds.terminal-output`
**Fix**: Stick to documented `ds.*` components or build custom ones

### ✅ Quick Fixes

- **Always test pages**: `curl http://127.0.0.1:8004/path/` before committing
- **Start simple**: Create minimal working version, then add complexity
- **Check nesting**: Count `-- ftd.column:` vs `-- end: ftd.column` statements
- **Use existing patterns**: Copy structure from working files like `/ftd/text.ftd`
- **Escape examples**: Use `\--` in all code blocks showing FTD syntax