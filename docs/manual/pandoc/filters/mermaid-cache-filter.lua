-- mermaid-cache-filter.lua — pandoc Lua filter for MERMAID_FILTER=skip mode.
-- Replaces every ```mermaid CodeBlock with `\includegraphics{<cache>/<sha>.svg}`,
-- where <sha> is sha256(block.text) and <cache> is from MERMAID_CACHE_DIR.
-- Hard-errors on cache-miss; warns on metadata mismatch / absent / malformed.
-- See ../../FOLLOWUPS.md (Closed: figures-cache-implementation) for design rationale.

-- Add this filter's directory to package.path so `require("sha256")` finds the
-- vendored module sitting alongside this file. Pandoc Lua's default cpath
-- only searches for .so files, so we extend the .lua path explicitly.
local filter_dir = debug.getinfo(1, "S").source:match("^@(.*)/[^/]+$")
if filter_dir then
  package.path = filter_dir .. "/?.lua;" .. package.path
end

local sha256 = require("sha256")

local cache_dir = os.getenv("MERMAID_CACHE_DIR") or "figures/cache"

local function check_metadata()
  local f = io.open(cache_dir .. "/cache-metadata.toml", "r")
  if not f then
    io.stderr:write("WARN: cache-metadata.toml absent or unparseable; cache provenance unknown\n")
    return
  end
  local content = f:read("*a")
  f:close()
  local cached = content:match('mermaid_cli_version%s*=%s*"([^"]+)"')
  if not cached then
    io.stderr:write("WARN: cache-metadata.toml absent or unparseable; cache provenance unknown\n")
    return
  end
  -- Best-effort version comparison; if mmdc absent (skip-mode build host),
  -- silently pass. The cached version is informational only here.
  local h = io.popen("mmdc --version 2>/dev/null")
  if h then
    local running = h:read("*l")
    h:close()
    if running and running ~= cached then
      io.stderr:write(string.format(
        "WARN: mmdc version mismatch (cached=%s, running=%s)\n",
        cached, running))
    end
  end
end

check_metadata()

function CodeBlock(block)
  if not block.classes:includes("mermaid") then return nil end
  local sha = sha256.hex(block.text)
  local pdf_path = cache_dir .. "/" .. sha .. ".pdf"
  local f = io.open(pdf_path, "r")
  if not f then
    io.stderr:write(string.format(
      "ERROR: missing cache entry %s.pdf for mermaid block (regenerate via `make figures-cache`)\n",
      sha))
    os.exit(1)
  end
  f:close()
  -- Emit \includegraphics with the .pdf cache entry: xelatex embeds PDFs
  -- natively (no svg.sty / inkscape conversion needed). Cache files render
  -- via mmdc -o *.pdf in the regen helper.
  return pandoc.RawBlock(
    "latex",
    string.format("\\includegraphics[width=\\textwidth]{%s}", pdf_path)
  )
end
