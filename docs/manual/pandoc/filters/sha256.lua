-- sha256.lua — pure-Lua SHA-256 for the figures-cache mermaid filter.
-- Source: derived from FIPS PUB 180-4 §6.2 (SHA-256 algorithm) reference
-- implementation; bitwise operations require Lua 5.3+. Pandoc ships Lua 5.4,
-- so the `>>`, `<<`, `&`, `|`, `~` operators are available.
-- Public domain. No external dependency.
--
-- API: local sha256 = require("sha256"); sha256.hex(string) -> hex_string
-- Byte-alignment contract (must match the Python regen helper):
-- input is the literal text between (and excluding) the ```mermaid and
-- closing ``` fences, no leading or trailing newline injected.

local M = {}

local K = {
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
  0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
  0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
  0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
  0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
  0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
  0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
  0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
  0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
}

local MASK32 = 0xffffffff

local function rotr(x, n)
  return ((x >> n) | (x << (32 - n))) & MASK32
end

local function preprocess(msg)
  local L = #msg
  -- 1-bit (0x80), then zeros, then 64-bit big-endian length-in-bits
  local pad = string.char(0x80)
  -- (L + 1 + k) ≡ 56 (mod 64), k ≥ 0
  local k = (56 - (L + 1)) % 64
  pad = pad .. string.rep("\0", k)
  local bits = L * 8
  -- 64-bit big-endian length
  local hi = (bits >> 32) & MASK32
  local lo = bits & MASK32
  pad = pad .. string.pack(">I4I4", hi, lo)
  return msg .. pad
end

local function compress(H, chunk)
  local W = {}
  for t = 0, 15 do
    W[t] = string.unpack(">I4", chunk, 1 + t * 4)
  end
  for t = 16, 63 do
    local s0 = rotr(W[t-15], 7) ~ rotr(W[t-15], 18) ~ (W[t-15] >> 3)
    local s1 = rotr(W[t-2], 17) ~ rotr(W[t-2], 19)  ~ (W[t-2] >> 10)
    W[t] = (W[t-16] + s0 + W[t-7] + s1) & MASK32
  end

  local a, b, c, d, e, f, g, h =
    H[1], H[2], H[3], H[4], H[5], H[6], H[7], H[8]

  for t = 0, 63 do
    local S1 = rotr(e, 6) ~ rotr(e, 11) ~ rotr(e, 25)
    local ch = (e & f) ~ ((~e) & g & MASK32)
    local T1 = (h + S1 + ch + K[t+1] + W[t]) & MASK32
    local S0 = rotr(a, 2) ~ rotr(a, 13) ~ rotr(a, 22)
    local mj = (a & b) ~ (a & c) ~ (b & c)
    local T2 = (S0 + mj) & MASK32
    h = g
    g = f
    f = e
    e = (d + T1) & MASK32
    d = c
    c = b
    b = a
    a = (T1 + T2) & MASK32
  end

  H[1] = (H[1] + a) & MASK32
  H[2] = (H[2] + b) & MASK32
  H[3] = (H[3] + c) & MASK32
  H[4] = (H[4] + d) & MASK32
  H[5] = (H[5] + e) & MASK32
  H[6] = (H[6] + f) & MASK32
  H[7] = (H[7] + g) & MASK32
  H[8] = (H[8] + h) & MASK32
end

function M.hex(msg)
  local padded = preprocess(msg)
  local H = {
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
  }
  for i = 1, #padded, 64 do
    compress(H, padded:sub(i, i + 63))
  end
  return string.format("%08x%08x%08x%08x%08x%08x%08x%08x",
    H[1], H[2], H[3], H[4], H[5], H[6], H[7], H[8])
end

return M
