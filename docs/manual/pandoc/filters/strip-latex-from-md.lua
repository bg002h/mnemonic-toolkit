-- strip-latex-from-md.lua
--
-- Removes raw-LaTeX inlines and blocks from the AST so the markdown
-- render path emits clean GFM. PDF render path does NOT load this filter.
--
-- Specifically targets:
--   * inline `\index{...}` directives (single backslash command + braces)
--   * `\begin{mdframed}`/`\end{mdframed}` and similar that may have leaked
--   * any RawInline / RawBlock with format == "tex" or "latex"

function RawInline(elem)
  if elem.format == "tex" or elem.format == "latex" then
    return {}
  end
  return nil
end

function RawBlock(elem)
  if elem.format == "tex" or elem.format == "latex" then
    return {}
  end
  return nil
end
