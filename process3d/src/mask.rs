//! Compressed bit masks.

/// A single compressed “word” in our run‐length encoding.
///
/// - `ZeroRun(n)`:   represents `n` consecutive words of all‐zeros (`0x0000…`)
/// - `OneRun(n)`:    represents `n` consecutive words of all‐ones  (`0xFFFF…`)
/// - `Literal(w)`:   represents exactly one word with arbitrary bits
#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    /// Repeat `0x0000...`.
    ZeroRun(usize),
    /// Repeat `0x1111...`.
    OneRun(usize),
    /// Non-uniform `u64` bit mask.
    Literal(u64),
}

/// A sequence of `u64` masks, compressed by run‐length encoding.
#[derive(Debug, Clone)]
pub struct CompressedMasks {
    segments: Vec<Segment>,
    // Total number of words represented (kept in sync with pushes).
    len_words: usize,
}

impl CompressedMasks {
    /// Create an empty sequence.
    pub fn new() -> Self {
        CompressedMasks {
            segments: Vec::new(),
            len_words: 0,
        }
    }

    /// Clearn the content.
    pub fn clear(&mut self) {
        self.segments.clear();
        self.len_words = 0;
    }

    /// Push a new 64‐bit mask on the end, merging into the last run if possible.
    pub fn push(&mut self, word: u64) {
        use Segment::*;
        self.len_words += 1;
        match self.segments.last_mut() {
            // extend an existing zero‐run
            Some(ZeroRun(n)) if word == 0 => *n += 1,
            // extend an existing one‐run
            Some(OneRun(n)) if word == !0 => *n += 1,
            // otherwise, we need a new segment
            _ => self.segments.push(match word {
                0 => ZeroRun(1),
                0xffffffffffffffff => OneRun(1),
                _ => Literal(word),
            })
        }
    }

    /// Get the uncompressed length (number of `u64` words).
    pub fn len(&self) -> usize {
        self.len_words
    }

    /// Counts the total number of ones.
    pub fn count_ones(&self) -> u64 {
        let mut sum: u64 = 0;
        for seg in &self.segments {
            match *seg {
                Segment::ZeroRun(_) => {}
                Segment::OneRun(count) => sum += (64 * count) as u64,
                Segment::Literal(w) => sum += w.count_ones() as u64,
            }
        }
        sum
    }

    /// Returns the word at position `i`, decompressing on the fly.
    pub fn get(&self, mut i: usize) -> Option<u64> {
        if i >= self.len_words {
            return None;
        }
        for seg in &self.segments {
            match *seg {
                Segment::ZeroRun(count) if i < count => return Some(0),
                Segment::ZeroRun(count) => { i -= count; }
                Segment::OneRun(count)  if i < count => return Some(!0),
                Segment::OneRun(count)  => { i -= count; }
                Segment::Literal(w)     if i == 0    => return Some(w),
                Segment::Literal(_)     => { i -= 1; }
            }
        }
        // should never reach here if len_words is correct
        None
    }

    /// Iterate through decompressed words, skipping zero runs.
    pub fn iter(&self) -> impl Iterator<Item = (usize, u64)> + Clone {
        self.segments.iter()
        .scan(0usize, |offset, seg| {
                let start = *offset;
                let count = match *seg {
                    Segment::ZeroRun(c) => c,
                    Segment::OneRun(c)  => c,
                    Segment::Literal(_) => 1,
                };
                *offset += count;
                Some((start, seg))
            })
        .filter(|(_, seg)| if let Segment::ZeroRun(_) = *seg {false} else {true})
        .flat_map(|(i, seg)| match *seg {
            Segment::ZeroRun(count) => std::iter::repeat(0).take(count),
            Segment::OneRun(count)  => std::iter::repeat(0xffffffffffffffff).take(count),
            Segment::Literal(w)     => std::iter::repeat(w).take(1),
        }.enumerate().map(move |(j, m)| (i + j, m)))
    }
}
