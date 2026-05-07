-- primer-box.lua
--
-- Renders fenced div blocks of class `primer` (and `danger`) into the
-- right output for each format:
--
--   :::primer    →  PDF: \begin{primerbox}...\end{primerbox}
--                   md:  blockquote with a "Background — " prefix
--   :::danger    →  PDF: \begin{dangerbox}...\end{dangerbox}
--                   md:  blockquote with a "DANGER — " prefix
--
-- This filter runs in BOTH the markdown and PDF pipelines. The split
-- happens by inspecting FORMAT.

function Div(el)
  local is_primer = el.classes:includes("primer")
  local is_danger = el.classes:includes("danger")

  if not (is_primer or is_danger) then
    return nil
  end

  local class = is_primer and "primerbox" or "dangerbox"
  local md_prefix = is_primer and "**Background.** " or "**DANGER.** "

  if FORMAT:match("latex") then
    local out = {
      pandoc.RawBlock("latex", "\\begin{" .. class .. "}"),
    }
    for _, b in ipairs(el.content) do
      table.insert(out, b)
    end
    table.insert(out, pandoc.RawBlock("latex", "\\end{" .. class .. "}"))
    return out
  else
    -- Wrap the contents in a BlockQuote, prepending the prefix to the
    -- first inline-bearing block (Para or Plain) it finds. If the div
    -- has none, just emit a BlockQuote with the prefix as a Para.
    local prefix_inline = pandoc.Strong({ pandoc.Str(md_prefix:match("^(.-)%s$") or md_prefix) })
    local prefixed = false
    local body = {}
    for _, b in ipairs(el.content) do
      if not prefixed and (b.t == "Para" or b.t == "Plain") then
        local new_inlines = { prefix_inline, pandoc.Space() }
        for _, i in ipairs(b.content) do
          table.insert(new_inlines, i)
        end
        if b.t == "Para" then
          table.insert(body, pandoc.Para(new_inlines))
        else
          table.insert(body, pandoc.Plain(new_inlines))
        end
        prefixed = true
      else
        table.insert(body, b)
      end
    end
    if not prefixed then
      table.insert(body, 1, pandoc.Para({ prefix_inline }))
    end
    return pandoc.BlockQuote(body)
  end
end
