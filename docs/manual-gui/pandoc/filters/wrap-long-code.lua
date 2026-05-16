-- wrap-long-code.lua
--
-- Insert line-break opportunities into inline-Code AST nodes that
-- contain long unbreakable monospace runs (mk1q…, xpub6…, long file
-- paths, long identifiers). Emits \brktt{<text>} (defined in
-- pandoc/preamble.tex) which wraps in \texttt{\seqsplit{...}}.
--
-- We intentionally do NOT touch Header content, Caption content, or
-- any moving-argument context. Pandoc emits \texttt{...} inside
-- \section{}, \caption{}, \hypertarget{} and bookmark strings.
-- Redefining \texttt globally OR walking Headers here would re-introduce
-- the moving-argument hang we just escaped (xelatex infinite-loops when
-- a char-splitter macro is re-read from .aux/.toc/.out files).
--
-- For CodeBlock long-string overflows: NOT addressed here. fvextra's
-- breakanywhere is already enabled on the Highlighting environment
-- and partially handles those. Worst-case 150pt overflows from a
-- single >100-char mk1/xpub on a code line still happen and are
-- tracked in docs/manual/FOLLOWUPS.md.

local MIN_RUN          = 12  -- inline Code: chunk if any non-space run >= 12
local MIN_BLOCK_RUN    = 40  -- CodeBlock: chunk lines with a non-space run >= 40
local BLOCK_CHUNK_SIZE = 64  -- CodeBlock: insert a newline every N chars in long runs

-- Detect whether the text has any contiguous non-space run >= MIN_RUN chars.
local function has_long_run(s)
  for run in s:gmatch('%S+') do
    if #run >= MIN_RUN then return true end
  end
  return false
end

-- Same but with an explicit minimum (used by CodeBlock with MIN_BLOCK_RUN).
local function has_long_run_min(s, min)
  for run in s:gmatch('%S+') do
    if #run >= min then return true end
  end
  return false
end

-- Escape characters that need escaping inside \texttt{...} / \brktt{...}.
-- Conservative: \texttt content goes through normal TeX tokenization,
-- so backslash, braces, ampersand, percent, dollar, hash, underscore,
-- caret, and tilde all need handling.
local function tex_escape(s)
  s = s:gsub('\\', '\\textbackslash{}')
  s = s:gsub('~',  '\\textasciitilde{}')
  s = s:gsub('%^', '\\textasciicircum{}')
  s = s:gsub('([%%&%$#%_%{%}])', '\\%1')
  return s
end

-- Topdown traversal lets us return el, false from Header / Caption to
-- prevent pandoc from walking into their inline children. That's what
-- keeps \brktt (and therefore \seqsplit) out of moving-argument contexts
-- like \section{}, \caption{}, and \hypertarget{} where seqsplit's
-- \futurelet machinery cannot survive .aux/.toc/.out re-reads.
--
-- Format gate: \brktt is a LaTeX-only macro. Emitting it via
-- pandoc.RawInline('latex', ...) is correct under the PDF pipeline
-- (xelatex consumes the raw-LaTeX inline + \seqsplit-wraps the token).
-- Under the HTML / GFM pipelines, raw-LaTeX inlines are silently
-- dropped by the html5 / gfm writers — which would strip every
-- ≥MIN_RUN-char code span from the rendered output. So the Code +
-- CodeBlock handlers must no-op outside latex. (Pattern mirrors
-- primer-box.lua:28 — the project-canonical format-gate idiom.) The
-- HTML render gets line-break behavior for free from the browser's
-- <code> wrapping; the CodeBlock-chunking is also LaTeX-only because
-- mid-token line breaks change visible output.
return {
  {
    traverse = 'topdown',

    -- Block recursion into headers and captions.
    Header = function(el) return el, false end,

    -- Process inline `Code` nodes only. Skip if no long unbreakable run.
    -- Skip entirely on non-LaTeX writers (HTML / GFM); raw-LaTeX inlines
    -- would be silently dropped, stripping every long backticked token
    -- from the rendered output (see file header for full rationale).
    Code = function(el)
      if not FORMAT:match("latex") then return nil end
      if not has_long_run(el.text) then return nil end
      local escaped = tex_escape(el.text)
      return pandoc.RawInline('latex', '\\brktt{' .. escaped .. '}')
    end,

    -- CodeBlock: walk lines and chunk any non-space run >= MIN_BLOCK_RUN
    -- with a real newline every BLOCK_CHUNK_SIZE chars. fvextra's
    -- Highlighting environment honors real newlines in Verbatim content
    -- (each becomes a wrap point), so the long mk1q…/xpub6… single-line
    -- tokens that overflow by 150pt+ get split into multiple visible
    -- wrapped lines instead. LaTeX-only: HTML's <pre><code> wraps
    -- naturally and mid-token line breaks would change visible output.
    CodeBlock = function(el)
      if not FORMAT:match("latex") then return nil end
      local changed = false
      local out_lines = {}
      for line in (el.text .. '\n'):gmatch('(.-)\n') do
        if not has_long_run_min(line, MIN_BLOCK_RUN) then
          table.insert(out_lines, line)
        else
          -- Walk the line, chunking any long non-space run.
          local out = {}
          local i = 1
          while i <= #line do
            local ws_start, ws_end = line:find('%s+', i)
            local run_end = (ws_start and ws_start - 1) or #line
            if ws_start == i then
              table.insert(out, line:sub(ws_start, ws_end))
              i = ws_end + 1
            else
              local run = line:sub(i, run_end)
              if #run >= MIN_BLOCK_RUN then
                local j = 1
                while j <= #run do
                  table.insert(out, run:sub(j, j + BLOCK_CHUNK_SIZE - 1))
                  j = j + BLOCK_CHUNK_SIZE
                  if j <= #run then table.insert(out, '\n') end
                end
                changed = true
              else
                table.insert(out, run)
              end
              i = run_end + 1
            end
          end
          table.insert(out_lines, table.concat(out))
        end
      end
      if not changed then return nil end
      el.text = table.concat(out_lines, '\n')
      return el
    end,
  },
}
