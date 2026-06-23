-- include-transcript.lua — pandoc Lua filter for build-time transcript inclusion.
--
-- A fenced code block carrying `include="<stem>.out"` has its body REPLACED by
-- the named file's contents, read from the transcripts root (env TRANSCRIPTS_DIR,
-- absolute). An optional `lines="N-M"` attribute selects a 1-based inclusive
-- contiguous line range (open-ended `lines="N-"` = "line N to EOF"). The
-- include=/lines= attributes are dropped so downstream filters/writers see a
-- clean code block.
--
-- Composed with the verify-examples runner (which gates .out == binary), this
-- makes prose == .out structural — the rendered fence body comes from the golden
-- by construction, not by hand-paste.
--
-- FAIL-CLOSED: a missing TRANSCRIPTS_DIR, a missing include target, a malformed
-- `lines=` spec, or an out-of-range range each writes a FATAL diagnostic to
-- stderr and exits 1 — the build FAILS rather than emitting a silent empty or
-- truncated fence.
--
-- Rationale for the env-var path (mirrors mermaid-cache-filter.lua's
-- MERMAID_CACHE_DIR): pandoc may run from any cwd, so a relative path would
-- silently mis-resolve. The Makefile exports TRANSCRIPTS_DIR := $(TRANSCRIPTS).
--
-- See ../../FOLLOWUPS.md / design/SPEC_cycleC_P0a_mechanism for design rationale.

local function fatal(msg)
  io.stderr:write("[include-transcript] FATAL: " .. msg .. "\n")
  os.exit(1)
end

-- Read the whole file, or fail-closed if it is missing/unreadable.
local function read_file(path, fence_include)
  local f = io.open(path, "r")
  if not f then
    fatal(string.format("include target missing: %s (fence include=\"%s\")",
      path, fence_include))
  end
  local content = f:read("*a")
  f:close()
  return content
end

-- Split a string into a line array. Exactly one trailing newline (the canonical
-- POSIX line terminator on the last line) is stripped first so an N-line file
-- yields N entries — never a phantom final empty line.
local function split_lines(content)
  -- Strip exactly ONE trailing "\n" (the last line's terminator) if present.
  if content:sub(-1) == "\n" then
    content = content:sub(1, -2)
  end
  local lines = {}
  -- gmatch with the explicit trailing newline captures every line including a
  -- final non-newline-terminated one. After the single strip above this yields
  -- one entry per content line.
  for line in (content .. "\n"):gmatch("(.-)\n") do
    table.insert(lines, line)
  end
  return lines
end

-- Parse a `lines="N-M"` (or open-ended "N-") spec into start/stop integers.
-- stop == nil means "to EOF". Fail-closed on any malformed spec.
local function parse_range(spec, fence_include)
  -- Closed range "N-M".
  local lo, hi = spec:match("^(%d+)%-(%d+)$")
  if lo then
    return tonumber(lo), tonumber(hi)
  end
  -- Open-ended range "N-".
  local lo_open = spec:match("^(%d+)%-$")
  if lo_open then
    return tonumber(lo_open), nil
  end
  fatal(string.format(
    "malformed lines= spec: %q (expected \"N-M\" or \"N-\") (fence include=\"%s\")",
    spec, fence_include))
end

function CodeBlock(el)
  local include = el.attributes["include"]
  if not include then return nil end

  local dir = os.getenv("TRANSCRIPTS_DIR")
  if not dir or dir == "" then
    fatal(string.format(
      "TRANSCRIPTS_DIR env var unset or empty (fence include=\"%s\")", include))
  end

  local path = dir .. "/" .. include
  local content = read_file(path, include)
  local lines = split_lines(content)

  local body
  local range = el.attributes["lines"]
  if range then
    local lo, hi = parse_range(range, include)
    if hi == nil then hi = #lines end
    if lo < 1 or lo > #lines or hi < lo or hi > #lines then
      fatal(string.format(
        "lines=\"%s\" out of range: file %s has %d line(s) (fence include=\"%s\")",
        range, path, #lines, include))
    end
    local sel = {}
    for i = lo, hi do
      table.insert(sel, lines[i])
    end
    body = table.concat(sel, "\n")
  else
    -- Whole-file include: join all lines (the single trailing newline was
    -- already stripped in split_lines), so the fence body has no spurious
    -- blank last line.
    body = table.concat(lines, "\n")
  end

  -- Drop the include= / lines= attributes; preserve all others, plus identity
  -- and classes, so downstream filters/writers see a clean code block.
  local attrs = {}
  for k, v in pairs(el.attributes) do
    if k ~= "include" and k ~= "lines" then
      attrs[k] = v
    end
  end

  return pandoc.CodeBlock(body, pandoc.Attr(el.identifier, el.classes, attrs))
end
