use super::macros::*;
use crate::Result;
use std::fmt;
//use faster::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Btrit {
    hi: u8,
    lo: u8,
}

impl Btrit {
    pub fn new(h: u8, l: u8) -> Btrit {
        Btrit { hi: h, lo: l }
    }

    #[inline]
    pub fn from_trit(&mut self, t: Trit) {
        self.hi = ((t & 0x2) >> 1) ^ (t & 0x1);
        self.lo = t & 0x1;
    }

    #[inline]
    pub fn to_trit(&self) -> Trit {
        (self.hi ^ self.lo) << 1 | self.lo
    }
}

#[derive(Clone)]
pub struct Stroika {
    num_rounds: usize,
    state: Vec<Btrit>,
}

impl Default for Stroika {
    fn default() -> Stroika {
        Stroika {
            num_rounds: NUM_ROUNDS,
            state: vec![Btrit::new(0, 0); STATE_SIZE],
        }
    }
}

impl fmt::Debug for Stroika {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Stroika: [rounds: [{}], state: {:?}",
            self.num_rounds, self.state,
        )
    }
}

impl Stroika {
    pub fn new(num_rounds: usize) -> Result<Stroika> {
        let mut stroika = Stroika::default();
        stroika.num_rounds = num_rounds;
        Ok(stroika)
    }

    pub fn state(&self) -> &Vec<Btrit> {
        &self.state
    }

    pub fn reset(&mut self) {
        self.state = vec![Btrit::new(0, 0); STATE_SIZE];
    }

    pub fn absorb(&mut self, message: &[Trit]) {
        let mut message_length = message.len();
        let mut message_idx = 0;
        let mut trit_idx = 0;
        let mut bmessage = vec![Btrit::new(0, 0); message_length];
        
        for idx in 0..message_length {
            bmessage[idx].from_trit(message[idx]);
        }

        while message_length >= TROIKA_RATE {
            // Copy message block over the state
            for trit_idx in 0..TROIKA_RATE {
                self.state[trit_idx] = bmessage[message_idx + trit_idx];
            }
            self.permutation();
            message_length -= TROIKA_RATE;
            message_idx += TROIKA_RATE;
        }

        // Pad last block
        let mut last_block = vec![Btrit::new(0, 0); TROIKA_RATE];

        // Copy over last incomplete message block
        for _ in 0..message_length {
            last_block[trit_idx] = bmessage[trit_idx];
            trit_idx += 1;
        }

        // Apply padding
        last_block[trit_idx] = Btrit {
            hi: PADDING,
            lo: PADDING,
        };

        // Insert last message block
        for trit_idx in 0..TROIKA_RATE {
            self.state[trit_idx] = last_block[trit_idx];
        }
    }

    pub fn squeeze(&mut self, hash: &mut [Trit]) {
        let mut hash_length = hash.len();
        let mut hash_idx = 0;
        let mut bhash = vec![Btrit::new(0, 0); hash_length];
        
        for idx in 0..hash_length {
            bhash[idx].from_trit(hash[idx]);
        }

        while hash_length >= TROIKA_RATE {
            self.permutation();
            // Extract rate output
            for trit_idx in 0..TROIKA_RATE {
                bhash[hash_idx + trit_idx] = self.state[trit_idx];
            }
            hash_idx += TROIKA_RATE;
            hash_length -= TROIKA_RATE;
        }

        // Check if there is a last incomplete block
        if hash_length % TROIKA_RATE != 0 {
            self.permutation();
            for trit_idx in 0..hash_length {
                bhash[trit_idx] = self.state[trit_idx];
            }
        }

        for idx in 0..hash.len() {
            hash[idx] = bhash[idx].to_trit();
        }
    }

    pub fn permutation(&mut self) {
        assert!(self.num_rounds <= NUM_ROUNDS);

        for round in 0..self.num_rounds {
            self.sub_trytes();
            self.shift_rows();
            self.shift_lanes();
            self.add_column_parity();
            self.add_round_constant(round);
        }
    }

    fn sub_trytes(&mut self) {
        /*for sbox_idx in 0..NUM_SBOXES {
            let sbox_input = 9 * self.state[3 * sbox_idx].to_trit()
                + 3 * self.state[3 * sbox_idx + 1].to_trit()
                + self.state[3 * sbox_idx + 2].to_trit();
            let mut sbox_output = SBOX_LOOKUP[sbox_input as usize];
            self.state[3 * sbox_idx + 2].from_trit(sbox_output % 3);
            sbox_output /= 3;
            self.state[3 * sbox_idx + 1].from_trit(sbox_output % 3);
            sbox_output /= 3;
            self.state[3 * sbox_idx].from_trit(sbox_output % 3);
        }*/
        let mut i = 0;
        while i < STATE_SIZE {
            self.simd_sbox(i);
            self.simd_sbox(i + 3);
            self.simd_sbox(i + 6);
            i += 9;
        }
    }

    #[inline]
    fn simd_sbox(&mut self, i: usize) {
        let (a, b, c) = (self.state[i], self.state[i + 1], self.state[i + 2]);
        let (mut x, mut y, mut z) = (Btrit::new(0, 0), Btrit::new(0, 0), Btrit::new(0, 0));

        x.hi = (((b.hi & c.hi)) | ((c.hi & !a.lo)) | ((a.lo ^ c.hi) & (b.hi))) & 0x1;
        x.lo = (((a.lo ^ b.lo) & (b.hi & c.hi)) | ((c.hi ^ c.lo) & (!a.hi & !b.hi)) | ((a.lo ^ b.hi ^ b.lo ^ c.hi) & (a.hi & c.lo)) | ((a.lo ^ c.hi) & (b.lo))) & 0x1;
	    y.hi = ((!(a.hi ^ b.lo ^ c.lo) & (!a.lo)) | ((a.hi ^ b.lo ^ c.hi ^ c.lo) & (!a.lo)) | ((b.hi ^ c.hi) & (!a.lo)) | (!(a.lo ^ c.hi) & (b.hi))) & 0x1;
	    y.lo = (((a.hi ^ a.lo) & (!b.hi)) | ((a.hi ^ b.hi) & (!a.lo & !c.hi)) | ((a.lo ^ b.lo ^ c.lo) & (a.hi & b.hi & c.hi))) & 0x1;
	    z.hi = (((a.hi ^ b.lo ^ c.lo) & (c.hi)) | ((a.hi ^ a.lo ^ b.lo ^ c.lo) & (c.hi)) | ((b.hi ^ c.hi) & (!a.lo))) & 0x1;
	    z.lo = (((a.lo & c.lo)) | (!(a.hi ^ b.lo ^ c.hi) & (b.hi & !a.lo & !c.lo)) | ((c.lo & !b.hi))) & 0x1;

        self.state[i] = x;
        self.state[i + 1] = y;
        self.state[i + 2] = z;
    }

    fn shift_rows(&mut self) {
        let mut new_state = vec![Btrit::new(0, 0); STATE_SIZE];

        for slice in 0..SLICES {
            for row in 0..ROWS {
                for col in 0..COLUMNS {
                    let old_idx = SLICESIZE * slice + COLUMNS * row + col;
                    let new_idx = SLICESIZE * slice
                        + COLUMNS * row
                        + (col + 3 * SHIFT_ROWS_PARAM[row]) % COLUMNS;
                    new_state[new_idx] = self.state[old_idx];
                }
            }
        }

        self.state = new_state;
    }

    fn shift_lanes(&mut self) {
        let mut new_state = vec![Btrit::new(0, 0); STATE_SIZE];

        for slice in 0..SLICES {
            for row in 0..ROWS {
                for col in 0..COLUMNS {
                    let old_idx = SLICESIZE * slice + COLUMNS * row + col;
                    let new_slice = (slice + SHIFT_LANES_PARAM[col + COLUMNS * row]) % SLICES;
                    let new_idx = SLICESIZE * (new_slice) + COLUMNS * row + col;
                    new_state[new_idx] = self.state[old_idx];
                }
            }
        }

        self.state = new_state;
    }

    fn add_column_parity(&mut self) {
        let mut parity = [0u8; SLICES * COLUMNS];

        // First compute parity for each column
        for slice in 0..SLICES {
            for col in 0..COLUMNS {
                let mut col_sum = 0;
                for row in 0..ROWS {
                    col_sum += self.state[SLICESIZE * slice + COLUMNS * row + col].to_trit();
                }
                parity[COLUMNS * slice + col] = col_sum % 3;
            }
        }

        // Add parity
        let mut tmp;
        for slice in 0..SLICES {
            for row in 0..ROWS {
                for col in 0..COLUMNS {
                    let idx = SLICESIZE * slice + COLUMNS * row + col;
                    let sum_to_add = parity[(col + 8) % 9 + COLUMNS * slice]
                        + parity[(col + 1) % 9 + COLUMNS * ((slice + 1) % SLICES)];
                    tmp = self.state[idx].to_trit();
                    self.state[idx].from_trit((tmp + sum_to_add) % 3);
                }
            }
        }
    }

    fn add_round_constant(&mut self, round: usize) {
        let mut tmp;
        for slice in 0..SLICES {
            for col in 0..COLUMNS {
                let idx = SLICESIZE * slice + col;
                tmp = self.state[idx].to_trit();
                self.state[idx].from_trit((tmp + ROUND_CONSTANTS[round][slice * COLUMNS + col]) % 3);
            }
        }
    }
}

#[cfg(test)]
mod test_troika {
    use super::*;

    const HASH: [u8; 243] = [
        0, 2, 2, 1, 2, 1, 0, 1, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 2,
        1, 1, 1, 0, 1, 0, 2, 1, 0, 0, 0, 1, 2, 0, 2, 1, 0, 0, 2, 1, 1, 1, 1, 1, 2, 0, 1, 0, 2, 1,
        1, 2, 0, 1, 1, 1, 1, 1, 2, 2, 0, 0, 2, 2, 2, 2, 0, 0, 2, 2, 2, 1, 2, 2, 0, 2, 1, 1, 2, 1,
        1, 1, 2, 2, 1, 1, 0, 0, 0, 2, 2, 2, 0, 2, 1, 1, 1, 1, 0, 0, 1, 0, 2, 0, 2, 0, 2, 0, 0, 0,
        0, 1, 1, 1, 0, 2, 1, 1, 1, 0, 2, 0, 0, 1, 0, 1, 0, 2, 0, 2, 2, 0, 0, 2, 2, 0, 1, 2, 1, 0,
        0, 1, 2, 1, 1, 0, 0, 1, 1, 0, 2, 1, 1, 0, 1, 2, 0, 0, 0, 1, 2, 2, 1, 1, 1, 0, 0, 2, 0, 1,
        1, 2, 1, 1, 2, 1, 0, 1, 2, 2, 2, 2, 1, 2, 0, 2, 2, 1, 2, 1, 2, 1, 2, 2, 1, 1, 2, 0, 2, 1,
        0, 1, 1, 1, 0, 2, 2, 0, 0, 2, 0, 2, 0, 1, 2, 0, 0, 2, 2, 1, 1, 2, 0, 1, 0, 0, 0, 0, 2, 0,
        2, 2, 2,
    ];

    #[test]
    fn test_hash() {
        let mut stroika = Stroika::default();
        let mut output = [0u8; 243];
        let input = [0u8; 243];
        stroika.absorb(&input);
        stroika.squeeze(&mut output);

        assert!(
            output.iter().zip(HASH.iter()).all(|(a, b)| a == b),
            "Arrays are not equal"
        );
    }
}
