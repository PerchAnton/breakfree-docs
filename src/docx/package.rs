//! Original bounded ZIP adapter: stored/deflated entries, no extraction to disk.
use crate::Result;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
};
pub const MAX: usize = 32 * 1024 * 1024;
pub const PART_MAX: usize = 8 * 1024 * 1024;
pub type Parts = BTreeMap<String, Vec<u8>>;
fn u16_at(b: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(at..at + 2)
            .ok_or("Truncated ZIP")?
            .try_into()
            .unwrap(),
    ))
}
fn u32_at(b: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(at..at + 4)
            .ok_or("Truncated ZIP")?
            .try_into()
            .unwrap(),
    ))
}
fn safe(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 256
        && !name.starts_with('/')
        && !name.chars().any(char::is_control)
        && !name.contains(['\\', ':', '%', '\0'])
        && name
            .split('/')
            .all(|s| !s.is_empty() && s != "." && s != "..")
}
pub fn read(bytes: &[u8]) -> Result<Parts> {
    if bytes.len() > MAX || bytes.len() < 22 {
        return Err("DOCX package is truncated or exceeds 32 MiB.".into());
    }
    let eocd = (bytes.len().saturating_sub(65557)..=bytes.len() - 22)
        .rev()
        .find(|&i| {
            bytes[i..].starts_with(b"PK\x05\x06")
                && u16_at(bytes, i + 20).is_ok_and(|n| i + 22 + usize::from(n) == bytes.len())
        })
        .ok_or("Missing ZIP directory")?;
    if u16_at(bytes, eocd + 4)? != 0
        || u16_at(bytes, eocd + 6)? != 0
        || u16_at(bytes, eocd + 8)? != u16_at(bytes, eocd + 10)?
    {
        return Err("Split ZIP packages are unsupported.".into());
    }
    let count = usize::from(u16_at(bytes, eocd + 10)?);
    let size = u32_at(bytes, eocd + 12)? as usize;
    let start = u32_at(bytes, eocd + 16)? as usize;
    if count == 0 || count > 128 || start.checked_add(size) != Some(eocd) {
        return Err("ZIP64, excessive entries or invalid ZIP directory.".into());
    }
    let mut pos = start;
    let mut parts = Parts::new();
    let mut total = 0usize;
    let mut spans = Vec::new();
    for _ in 0..count {
        if !bytes
            .get(pos..)
            .is_some_and(|b| b.starts_with(b"PK\x01\x02"))
        {
            return Err("Invalid ZIP entry".into());
        }
        let flags = u16_at(bytes, pos + 8)?;
        let method = u16_at(bytes, pos + 10)?;
        let crc = u32_at(bytes, pos + 16)?;
        let compressed = u32_at(bytes, pos + 20)? as usize;
        let expanded = u32_at(bytes, pos + 24)? as usize;
        let nl = usize::from(u16_at(bytes, pos + 28)?);
        let el = usize::from(u16_at(bytes, pos + 30)?);
        let cl = usize::from(u16_at(bytes, pos + 32)?);
        let offset = u32_at(bytes, pos + 42)? as usize;
        if flags & !0x80e != 0
            || ![0, 8].contains(&method)
            || expanded > PART_MAX
            || u16_at(bytes, pos + 34)? != 0
        {
            return Err("Encrypted, oversized or unsupported ZIP entry.".into());
        }
        let name = std::str::from_utf8(
            bytes
                .get(pos + 46..pos + 46 + nl)
                .ok_or("Truncated ZIP name")?,
        )
        .map_err(|_| "Non-UTF8 ZIP path")?
        .to_owned();
        if !safe(&name) || parts.contains_key(&name) {
            return Err("Unsafe or duplicate ZIP path.".into());
        }
        pos = pos
            .checked_add(46 + nl + el + cl)
            .filter(|p| *p <= eocd)
            .ok_or("Invalid ZIP directory size")?;
        if !bytes
            .get(offset..)
            .is_some_and(|b| b.starts_with(b"PK\x03\x04"))
            || u16_at(bytes, offset + 6)? != flags
            || u16_at(bytes, offset + 8)? != method
        {
            return Err("ZIP header mismatch.".into());
        }
        let local_n = usize::from(u16_at(bytes, offset + 26)?);
        let local_e = usize::from(u16_at(bytes, offset + 28)?);
        if bytes.get(offset + 30..offset + 30 + local_n) != Some(name.as_bytes()) {
            return Err("ZIP local path mismatch.".into());
        }
        if flags & 8 == 0
            && (u32_at(bytes, offset + 14)? != crc
                || u32_at(bytes, offset + 18)? as usize != compressed
                || u32_at(bytes, offset + 22)? as usize != expanded)
        {
            return Err("ZIP local size/checksum mismatch.".into());
        }
        let begin = offset
            .checked_add(30 + local_n + local_e)
            .ok_or("Invalid ZIP offset")?;
        let end = begin
            .checked_add(compressed)
            .filter(|e| *e <= start)
            .ok_or("ZIP data overlaps directory")?;
        let entry_end = if flags & 8 != 0 {
            let at = if bytes.get(end..end + 4) == Some(b"PK\x07\x08") {
                end + 4
            } else {
                end
            };
            if u32_at(bytes, at)? != crc
                || u32_at(bytes, at + 4)? as usize != compressed
                || u32_at(bytes, at + 8)? as usize != expanded
                || at + 12 > start
            {
                return Err("Invalid ZIP data descriptor".into());
            }
            at + 12
        } else {
            end
        };
        if spans.iter().any(|&(a, b)| offset < b && entry_end > a) {
            return Err("Overlapping ZIP entries.".into());
        }
        spans.push((offset, entry_end));
        total = total
            .checked_add(expanded)
            .filter(|n| *n <= MAX)
            .ok_or("Expanded DOCX exceeds 32 MiB")?;
        let input = bytes.get(begin..end).ok_or("Invalid ZIP data range")?;
        let data = if method == 0 {
            input.to_vec()
        } else {
            let mut d = flate2::read::DeflateDecoder::new(input);
            let mut v = Vec::new();
            d.by_ref()
                .take((expanded + 1) as u64)
                .read_to_end(&mut v)
                .map_err(|e| format!("Invalid deflate: {e}"))?;
            if d.total_in() != compressed as u64 {
                return Err("Trailing compressed data.".into());
            }
            v
        };
        if data.len() != expanded || crc32fast::hash(&data) != crc {
            return Err("DOCX entry size/checksum mismatch.".into());
        }
        parts.insert(name, data);
    }
    if pos != eocd {
        return Err("Unexpected ZIP directory data.".into());
    }
    Ok(parts)
}
fn put16(v: &mut Vec<u8>, n: usize) {
    v.extend((n as u16).to_le_bytes());
}
fn put32(v: &mut Vec<u8>, n: usize) {
    v.extend((n as u32).to_le_bytes());
}
pub fn write(parts: &Parts) -> Result<Vec<u8>> {
    if parts.len() > 128 || parts.values().map(Vec::len).sum::<usize>() > MAX {
        return Err("DOCX package exceeds limits.".into());
    }
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, data) in parts {
        if !safe(name) || data.len() > PART_MAX {
            return Err("Invalid DOCX part.".into());
        }
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).map_err(|e| e.to_string())?;
        let packed = encoder.finish().map_err(|e| e.to_string())?;
        let crc = crc32fast::hash(data);
        let offset = out.len();
        out.extend(b"PK\x03\x04");
        for n in [20, 0x800, 8, 0, 33] {
            put16(&mut out, n);
        }
        put32(&mut out, crc as usize);
        put32(&mut out, packed.len());
        put32(&mut out, data.len());
        put16(&mut out, name.len());
        put16(&mut out, 0);
        out.extend(name.as_bytes());
        out.extend(&packed);
        directory.extend(b"PK\x01\x02");
        for n in [20, 20, 0x800, 8, 0, 33] {
            put16(&mut directory, n);
        }
        put32(&mut directory, crc as usize);
        put32(&mut directory, packed.len());
        put32(&mut directory, data.len());
        for n in [name.len(), 0, 0, 0, 0] {
            put16(&mut directory, n);
        }
        put32(&mut directory, 0);
        put32(&mut directory, offset);
        directory.extend(name.as_bytes());
    }
    let start = out.len();
    let size = directory.len();
    out.extend(directory);
    out.extend(b"PK\x05\x06");
    for n in [0, 0, parts.len(), parts.len()] {
        put16(&mut out, n);
    }
    put32(&mut out, size);
    put32(&mut out, start);
    put16(&mut out, 0);
    if out.len() > MAX {
        return Err("DOCX exceeds 32 MiB.".into());
    }
    Ok(out)
}
