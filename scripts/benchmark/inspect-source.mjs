import { createHash } from 'node:crypto';
import { closeSync, constants, fstatSync, lstatSync, openSync, readSync } from 'node:fs';
import { dirname, isAbsolute, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// An evidence detector, not a model-tool interception boundary. It recognizes
// literal values and common JS/JSON character escapes, not arbitrary encodings.
export function containsProtectedValue(text, values) {
  if (typeof text !== 'string' || !values.length) return false;
  const decoded = text.replace(/\\+(?:u([\da-f]{4})|x([\da-f]{2}))/gi,
    (_, unicode, hex) => String.fromCharCode(Number.parseInt(unicode ?? hex, 16)));
  return values.some(value => text.includes(value) || decoded.includes(value));
}

function maskLiterals(source) {
  // Mask strings (including complete template strings) and comments before
  // collecting identifiers. Never execute or import the inspected source.
  return source.replace(/\/\*[\s\S]*?\*\/|\/\/[^\r\n]*|'(?:\\[\s\S]|[^'\\])*'|"(?:\\[\s\S]|[^"\\])*"|`(?:\\[\s\S]|[^`\\])*`/g,
    value => value.replace(/[^\r\n]/g, ' '));
}

export function inspectSource(root, filename, symbol) {
  const base = resolve(root), target = resolve(base, filename), rel = relative(base, target);
  if (!rel || isAbsolute(rel) || rel === '..' || rel.startsWith('../') || rel.startsWith('..\\')) {
    throw new Error('Source must be inside the current checkout.');
  }
  for (let part = target; ; part = dirname(part)) {
    const stat = lstatSync(part);
    if (stat.isSymbolicLink()) throw new Error('Linked source is blocked.');
    if (part === base) break;
  }
  const stat = lstatSync(target);
  if (!stat.isFile() || stat.nlink !== 1 || stat.size > 65536) throw new Error('Source must be one regular file of at most 64 KiB.');
  if (symbol && !/^[A-Za-z_$][\w$]{0,63}$/.test(symbol)) throw new Error('Invalid function identifier.');
  const fd = openSync(target, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0));
  let bytes;
  try {
    const opened = fstatSync(fd);
    if (!opened.isFile() || opened.nlink !== 1 || opened.size > 65536 || opened.ino !== stat.ino || opened.dev !== stat.dev) {
      throw new Error('Source changed or is outside the bounded regular-file policy.');
    }
    const buffer = Buffer.alloc(65537);
    let length = 0, count;
    while (length < buffer.length && (count = readSync(fd, buffer, length, buffer.length-length, null)) > 0) length += count;
    if (length > 65536) throw new Error('Source exceeds 64 KiB.');
    bytes = buffer.subarray(0,length);
  } finally { closeSync(fd); }
  const source = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  const masked = maskLiterals(source);
  // A small lexical inspector must reject syntax it cannot safely distinguish
  // from literal contents. A full parser/authorized safer read is needed here.
  if (source.includes('${') || /[\/\\'"`]/.test(masked)) throw new Error('Unsupported source syntax for bounded metadata inspection.');
  const identifiers = pattern => [...new Set([...masked.matchAll(pattern)].map(match => match[1]))].sort();
  const fields = identifiers(/\b([A-Za-z_$][\w$]{0,63})\s*:/g);
  const functions = [];
  for (const match of masked.matchAll(/\b(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][\w$]{0,63})\s*\([^)]*\)\s*\{/g)) {
    if (symbol && match[1] !== symbol) continue;
    const start = match.index + match[0].length;
    let depth = 1, end = start;
    for (; end < masked.length && depth; end++) {
      if (masked[end] === '{') depth++;
      else if (masked[end] === '}') depth--;
    }
    if (depth) throw new Error('Function structure is incomplete.');
    const body = masked.slice(start, end - 1);
    const properties = [...new Set([...body.matchAll(/\.\s*([A-Za-z_$][\w$]{0,63})\b|\b([A-Za-z_$][\w$]{0,63})\s*:/g)]
      .map(item => item[1] ?? item[2]))].sort();
    const objectKeys = [...new Set([...body.matchAll(/\b([A-Za-z_$][\w$]{0,63})\s*:/g)].map(item => item[1]))].sort();
    functions.push({ name: match[1], exported: /\bexport\s/.test(match[0]), properties, objectKeys });
  }
  if (fields.length > 128 || functions.length > 32 || functions.some(item => item.properties.length > 128)) {
    throw new Error('Structural metadata exceeds the bounded result.');
  }
  if (symbol && !functions.length) throw new Error('Requested function metadata is unavailable.');
  return { sha256: createHash('sha256').update(bytes).digest('hex'), bytes: bytes.length, fields, functions,
    limits: 'Structural identifiers only; strings/comments, values, expressions and source snippets are not returned. This is lexical metadata, not semantic proof. Other shell commands are not intercepted.' };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (![3, 5].includes(process.argv.length) || (process.argv.length === 5 && process.argv[3] !== '--symbol')) {
      throw new Error('Usage: node inspect-source.mjs <relative-source> [--symbol <function>].');
    }
    process.stdout.write(JSON.stringify(inspectSource(process.cwd(), process.argv[2], process.argv[4]), null, 2) + '\n');
  } catch {
    // Filesystem/decoder errors may contain source data or private paths.
    process.stderr.write('SOURCE_INSPECTION_BLOCKED: no source content returned. Use a regular bounded UTF-8 source in this checkout and an available function identifier.\n');
    process.exitCode = 2;
  }
}
