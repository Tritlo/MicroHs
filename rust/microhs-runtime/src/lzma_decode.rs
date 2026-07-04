// Fallible copy of lzma-sdk-rs' raw decoder. The upstream API panics on malformed
// streams; MicroHs needs an EvalError, and release builds use panic=abort.

const NUM_LIT_STATES: u32 = 7;
const NUM_POS_STATES_MAX: usize = 16;
const NUM_STATES: usize = 12;
const BIT_MODEL_TOTAL: u32 = 1 << NUM_BIT_MODEL_TOTAL_BITS;
const END_POS_MODEL_INDEX: u32 = 14;
const MATCH_LEN_MIN: u32 = 2;
const NUM_ALIGN_BITS: u32 = 4;
const NUM_BIT_MODEL_TOTAL_BITS: u32 = 11;
const NUM_FULL_DISTANCES: u32 = 1 << (END_POS_MODEL_INDEX >> 1);
const NUM_LEN_TO_POS_STATES: u32 = 4;
const NUM_MOVE_BITS: u32 = 5;
const NUM_POS_SLOT_BITS: u32 = 6;
const PROB_INIT_VALUE: u16 = (BIT_MODEL_TOTAL >> 1) as u16;
const START_POS_MODEL_INDEX: u32 = 4;
const TOP_VALUE: u32 = 1 << 24;

const POS_SLOT_TABLE_LEN: usize = 1 << NUM_POS_SLOT_BITS;
const ALIGN_TABLE_LEN: usize = 1 << NUM_ALIGN_BITS;
const LEN_LOW_TABLE_LEN: usize = 8;
const LEN_HIGH_TABLE_LEN: usize = 256;

struct LenProbs {
    choice: u16,
    choice2: u16,
    low: [[u16; LEN_LOW_TABLE_LEN]; NUM_POS_STATES_MAX],
    mid: [[u16; LEN_LOW_TABLE_LEN]; NUM_POS_STATES_MAX],
    high: [u16; LEN_HIGH_TABLE_LEN],
}

impl LenProbs {
    fn new() -> Self {
        Self {
            choice: PROB_INIT_VALUE,
            choice2: PROB_INIT_VALUE,
            low: [[PROB_INIT_VALUE; LEN_LOW_TABLE_LEN]; NUM_POS_STATES_MAX],
            mid: [[PROB_INIT_VALUE; LEN_LOW_TABLE_LEN]; NUM_POS_STATES_MAX],
            high: [PROB_INIT_VALUE; LEN_HIGH_TABLE_LEN],
        }
    }
}

struct Probs {
    is_match: [[u16; NUM_POS_STATES_MAX]; NUM_STATES],
    is_rep: [u16; NUM_STATES],
    is_rep_g0: [u16; NUM_STATES],
    is_rep_g1: [u16; NUM_STATES],
    is_rep_g2: [u16; NUM_STATES],
    is_rep0_long: [[u16; NUM_POS_STATES_MAX]; NUM_STATES],
    pos_slot: [[u16; POS_SLOT_TABLE_LEN]; NUM_LEN_TO_POS_STATES as usize],
    spec_pos: [u16; NUM_FULL_DISTANCES as usize],
    align: [u16; ALIGN_TABLE_LEN],
    len: LenProbs,
    rep_len: LenProbs,
    literal: Vec<u16>,
}

impl Probs {
    fn new(lc: u32, lp: u32) -> Option<Self> {
        let literal_len = 0x300usize.checked_shl(lc.checked_add(lp)?)?;
        let mut literal = Vec::new();
        literal.try_reserve(literal_len).ok()?;
        literal.resize(literal_len, PROB_INIT_VALUE);
        Some(Self {
            is_match: [[PROB_INIT_VALUE; NUM_POS_STATES_MAX]; NUM_STATES],
            is_rep: [PROB_INIT_VALUE; NUM_STATES],
            is_rep_g0: [PROB_INIT_VALUE; NUM_STATES],
            is_rep_g1: [PROB_INIT_VALUE; NUM_STATES],
            is_rep_g2: [PROB_INIT_VALUE; NUM_STATES],
            is_rep0_long: [[PROB_INIT_VALUE; NUM_POS_STATES_MAX]; NUM_STATES],
            pos_slot: [[PROB_INIT_VALUE; POS_SLOT_TABLE_LEN]; NUM_LEN_TO_POS_STATES as usize],
            spec_pos: [PROB_INIT_VALUE; NUM_FULL_DISTANCES as usize],
            align: [PROB_INIT_VALUE; ALIGN_TABLE_LEN],
            len: LenProbs::new(),
            rep_len: LenProbs::new(),
            literal,
        })
    }
}

struct RangeDecoder<'a> {
    input: &'a [u8],
    pos: usize,
    range: u32,
    code: u32,
}

impl<'a> RangeDecoder<'a> {
    fn new(input: &'a [u8]) -> Self {
        let b = |i: usize| input.get(i).copied().unwrap_or(0);
        let code = (u32::from(b(1)) << 24)
            | (u32::from(b(2)) << 16)
            | (u32::from(b(3)) << 8)
            | u32::from(b(4));
        Self {
            input,
            pos: 5,
            range: 0xFFFF_FFFF,
            code,
        }
    }

    #[inline]
    fn next_byte(&mut self) -> u32 {
        let b = self.input.get(self.pos).copied().unwrap_or(0);
        self.pos += 1;
        u32::from(b)
    }

    #[inline]
    fn normalize(&mut self) {
        if self.range < TOP_VALUE {
            self.range <<= 8;
            self.code = (self.code << 8) | self.next_byte();
        }
    }

    #[inline]
    fn decode_bit(&mut self, prob: &mut u16) -> u32 {
        self.normalize();
        let ttt = u32::from(*prob);
        let bound = (self.range >> NUM_BIT_MODEL_TOTAL_BITS) * ttt;
        if self.code < bound {
            self.range = bound;
            *prob = (ttt + ((BIT_MODEL_TOTAL - ttt) >> NUM_MOVE_BITS)) as u16;
            0
        } else {
            self.range -= bound;
            self.code -= bound;
            *prob = (ttt - (ttt >> NUM_MOVE_BITS)) as u16;
            1
        }
    }

    fn decode_direct_bits(&mut self, n: u32) -> u32 {
        let mut result = 0u32;
        for _ in 0..n {
            self.normalize();
            self.range >>= 1;
            self.code = self.code.wrapping_sub(self.range);
            let t = 0u32.wrapping_sub(self.code >> 31);
            self.code = self.code.wrapping_add(self.range & t);
            result = (result << 1).wrapping_add(t.wrapping_add(1));
        }
        result
    }

    fn decode_tree(&mut self, probs: &mut [u16], num_bits: u32) -> Option<u32> {
        let mut m = 1u32;
        for _ in 0..num_bits {
            let b = self.decode_bit(probs.get_mut(m as usize)?);
            m = (m << 1) | b;
        }
        Some(m - (1 << num_bits))
    }

    fn decode_tree_reverse(&mut self, probs: &mut [u16], num_bits: u32) -> Option<u32> {
        let mut m = 1u32;
        let mut sym = 0u32;
        for i in 0..num_bits {
            let b = self.decode_bit(probs.get_mut(m as usize)?);
            m = (m << 1) | b;
            sym |= b << i;
        }
        Some(sym)
    }
}

fn decode_len(rc: &mut RangeDecoder, len: &mut LenProbs, pos_state: usize) -> Option<u32> {
    if rc.decode_bit(&mut len.choice) == 0 {
        rc.decode_tree(len.low.get_mut(pos_state)?, 3)
    } else if rc.decode_bit(&mut len.choice2) == 0 {
        Some(8 + rc.decode_tree(len.mid.get_mut(pos_state)?, 3)?)
    } else {
        Some(16 + rc.decode_tree(&mut len.high, 8)?)
    }
}

pub(crate) fn decode_raw_checked(input: &[u8], props: &[u8; 5], out_len: usize) -> Option<Vec<u8>> {
    let mut d = u32::from(props[0]);
    if d >= 9 * 5 * 5 {
        return None;
    }
    let lc = d % 9;
    d /= 9;
    let lp = d % 5;
    let pb = d / 5;
    let pb_mask = (1u32 << pb) - 1;
    let lp_mask = (1u32 << lp) - 1;

    let mut probs = Probs::new(lc, lp)?;
    let mut rc = RangeDecoder::new(input);

    let mut out = Vec::new();
    out.try_reserve(out_len).ok()?;
    let mut state: u32 = 0;
    let mut reps: [u32; 4] = [1, 1, 1, 1];

    while out.len() < out_len {
        let pos_state = (out.len() as u32 & pb_mask) as usize;

        if rc.decode_bit(probs.is_match.get_mut(state as usize)?.get_mut(pos_state)?) == 0 {
            let prev = if out.is_empty() {
                0u32
            } else {
                u32::from(*out.last()?)
            };
            let lit_state = (((out.len() as u32 & lp_mask) << lc) + (prev >> (8 - lc))) as usize;
            let base = 0x300usize.checked_mul(lit_state)?;
            let table = probs.literal.get_mut(base..base.checked_add(0x300)?)?;

            let symbol = if state < NUM_LIT_STATES {
                let mut sym = 1u32;
                while sym < 0x100 {
                    let b = rc.decode_bit(table.get_mut(sym as usize)?);
                    sym = (sym << 1) | b;
                }
                sym
            } else {
                let rep0 = reps[0] as usize;
                let start = out.len().checked_sub(rep0)?;
                let mut match_byte = u32::from(*out.get(start)?);
                let mut sym = 1u32;
                let mut offs = 0x100u32;
                while sym < 0x100 {
                    match_byte <<= 1;
                    let match_bit = offs;
                    offs &= match_byte;
                    let idx = (offs + match_bit + sym) as usize;
                    let b = rc.decode_bit(table.get_mut(idx)?);
                    sym = (sym << 1) | b;
                    if b == 0 {
                        offs ^= match_bit;
                    }
                }
                sym
            };

            out.push((symbol & 0xFF) as u8);
            state = if state < 4 {
                0
            } else if state < 10 {
                state - 3
            } else {
                state - 6
            };
            continue;
        }

        let len_symbol;
        if rc.decode_bit(probs.is_rep.get_mut(state as usize)?) == 0 {
            reps[3] = reps[2];
            reps[2] = reps[1];
            reps[1] = reps[0];
            len_symbol = decode_len(&mut rc, &mut probs.len, pos_state)?;
            state = if state < NUM_LIT_STATES { 7 } else { 10 };

            let len_to_pos = len_symbol.min(NUM_LEN_TO_POS_STATES - 1) as usize;
            let pos_slot =
                rc.decode_tree(probs.pos_slot.get_mut(len_to_pos)?, NUM_POS_SLOT_BITS)?;
            let dist = if pos_slot < START_POS_MODEL_INDEX {
                pos_slot
            } else {
                let num_direct = (pos_slot >> 1) - 1;
                let mut dist = 2 | (pos_slot & 1);
                if pos_slot < END_POS_MODEL_INDEX {
                    dist <<= num_direct;
                    let mut m = 1u32;
                    let mut node = dist + 1;
                    let mut nd = num_direct;
                    loop {
                        let b = rc.decode_bit(probs.spec_pos.get_mut(node as usize)?);
                        if b == 0 {
                            node = node.wrapping_add(m);
                            m = m.wrapping_add(m);
                        } else {
                            m = m.wrapping_add(m);
                            node = node.wrapping_add(m);
                        }
                        nd -= 1;
                        if nd == 0 {
                            break;
                        }
                    }
                    node.wrapping_sub(m)
                } else {
                    dist <<= num_direct;
                    let high = rc.decode_direct_bits(num_direct - NUM_ALIGN_BITS) << NUM_ALIGN_BITS;
                    let dist = dist.wrapping_add(high);
                    dist.wrapping_add(rc.decode_tree_reverse(&mut probs.align, NUM_ALIGN_BITS)?)
                }
            };

            if dist == 0xFFFF_FFFF {
                break;
            }
            reps[0] = dist.checked_add(1)?;
        } else {
            if rc.decode_bit(probs.is_rep_g0.get_mut(state as usize)?) == 0 {
                if rc.decode_bit(
                    probs
                        .is_rep0_long
                        .get_mut(state as usize)?
                        .get_mut(pos_state)?,
                ) == 0
                {
                    let rep0 = reps[0] as usize;
                    let start = out.len().checked_sub(rep0)?;
                    let b = *out.get(start)?;
                    out.push(b);
                    state = if state < NUM_LIT_STATES { 9 } else { 11 };
                    continue;
                }
            } else {
                let dist;
                if rc.decode_bit(probs.is_rep_g1.get_mut(state as usize)?) == 0 {
                    dist = reps[1];
                } else if rc.decode_bit(probs.is_rep_g2.get_mut(state as usize)?) == 0 {
                    dist = reps[2];
                    reps[2] = reps[1];
                } else {
                    dist = reps[3];
                    reps[3] = reps[2];
                    reps[2] = reps[1];
                }
                reps[1] = reps[0];
                reps[0] = dist;
            }
            len_symbol = decode_len(&mut rc, &mut probs.rep_len, pos_state)?;
            state = if state < NUM_LIT_STATES { 8 } else { 11 };
        }

        let len = (len_symbol + MATCH_LEN_MIN) as usize;
        let rep0 = reps[0] as usize;
        let copy = len.min(out_len - out.len());
        for _ in 0..copy {
            let start = out.len().checked_sub(rep0)?;
            let b = *out.get(start)?;
            out.push(b);
        }
    }

    Some(out)
}
