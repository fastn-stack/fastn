-- Function to set up the filetype with the syntax highlight for .ftd files
local function SetupFtdSyntax()
  vim.bo.filetype = 'ftd'

  vim.cmd([[
    " Component Declarations, with optional whitespace handling for nested components
    syntax match ComponentDeclaration "^\s*--\s\+\(\w\|[-.]\)\+" contained
    syntax match ComponentEnd "^\s*--\s*end:\s*\(\w\|[-.]\)\+" contained
    " syntax match ComponentDeclaration "^\s*--\s\+\w\+" contained
    " syntax match ComponentEnd "^\s*--\s\+end:\s\+\w\+" contained

    " Define a broader match for any line that could contain a key-value pair, if necessary
    syntax match ComponentLine "^\s*\w\+[\w\.\-$]*\s*:\s*.*" contains=ComponentKey

    " Match only the key part of a key:value pair
    syntax match ComponentKey "^\s*\(\w\|[-.]\)\+\ze:"

    " Comments: Adjusted patterns to ensure correct matching
    syntax match ComponentComment "^\s*;;.*" contained

    " Apply contains=ALL to ensure nested components and comments
    " are highlighted within parent components
    syntax region ComponentStart start=/^\s*--\s\+\w\+/ end=/^\s*--\s\+end:/ contains=ComponentDeclaration,ComponentEnd,ComponentKey,ComponentComment
    syntax region ComponentRegion start="pattern" end="pattern" contains=ComponentKey

    " Highlight links
    highlight link ComponentDeclaration Tag
    highlight link ComponentEnd PreProc
    highlight link ComponentKey Identifier
    highlight link ComponentComment Comment
  ]])
end

-- Set up autocommands to apply the custom syntax highlighting for .ftd files
vim.api.nvim_create_autocmd({ "BufRead", "BufNewFile" }, {
  pattern = "*.ftd",
  callback = SetupFtdSyntax,
})
