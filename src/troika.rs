use crate::macros::*;

pub fn troika(output: &mut [Trit], input: &[Trit]) {
    troika_var_rounds(output, input, NUM_ROUNDS);
}

pub fn troika_var_rounds(output: &mut [Trit], input: &[Trit], num_rounds: usize) {
    let mut state = [0u8; STATE_SIZE];

    troika_absorb(&mut state, TROIKA_RATE, input, num_rounds);
    troika_squeeze(output, TROIKA_RATE, &mut state, num_rounds);
}

fn troika_absorb(state: &mut [Trit], rate: usize, message: &[Trit], num_rounds: usize) {
    let mut message_length = message.len();
    let mut message_idx = 0;
    let mut trit_idx = 0;

    while message_length >= rate {
        // Copy message block over the state
        for trit_idx in 0..rate {
            state[trit_idx] = message[message_idx + trit_idx];
        }
        troika_permutation(state, num_rounds);
        message_length -= rate;
        message_idx += rate;
    }

    // Pad last block
    let mut last_block = vec![0u8; rate];
    

    // Copy over last incomplete message block
    for _ in 0..message_length {
        last_block[trit_idx] = message[trit_idx];
        trit_idx += 1;
    }

    // TODO: Check trit_idx is right here
    // Apply padding
    last_block[trit_idx] = PADDING;

    // Insert last message block
    for trit_idx in 0..rate {
        state[trit_idx] = last_block[trit_idx];
    }
}

fn troika_squeeze(hash: &mut [Trit], rate: usize, state: &mut [Trit], num_rounds: usize) {
    let mut hash_length = hash.len();
    let mut hash_idx = 0;

    while hash_length >= rate {
        troika_permutation(state, num_rounds);
        // Extract rate output
        for trit_idx in 0..rate {
            hash[hash_idx + trit_idx] = state[trit_idx];
        }
        hash_idx += rate;
        hash_length -= rate;
    }

    // Check if there is a last incomplete block
    if hash_length % rate != 0 {
        troika_permutation(state, num_rounds);
        for trit_idx in 0..hash_length {
            hash[trit_idx] = state[trit_idx];
        }
    }
}


fn troika_permutation(state: &mut [Trit], num_rounds: usize) {
    assert!(num_rounds <= NUM_ROUNDS);

    for round in 0..num_rounds {
        sub_trytes(state);
        shift_rows(state);
        shift_lanes(state);
        add_column_parity(state);
        add_round_constant(state, round);
    }
}

fn sub_trytes(state: &mut [Trit]) {
    for sbox_idx in 0..NUM_SBOXES {
        let sbox_input = 9 * state[3 * sbox_idx] + 3 * state[3 * sbox_idx + 1] + state[3 * sbox_idx + 2];
        let mut sbox_output = SBOX_LOOKUP[sbox_input as usize];
        state[3 * sbox_idx + 2] = sbox_output % 3;
        sbox_output /= 3;
        state[3 * sbox_idx + 1] = sbox_output % 3;
        sbox_output /= 3;
        state[3 * sbox_idx] = sbox_output % 3;
    }
}

fn shift_rows(state: &mut [Trit]) {
    let mut new_state = [0u8; STATE_SIZE];

    for slice in 0..SLICES {
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let old_idx = SLICESIZE * slice + COLUMNS * row + col;
                let new_idx = SLICESIZE * slice + COLUMNS * row + (col + 3 * SHIFT_ROWS_PARAM[row]) % COLUMNS;
                new_state[new_idx] = state[old_idx];
            }
        }
    }

    state.copy_from_slice(&new_state[..]);
}

fn shift_lanes(state: &mut [Trit]) {
    let mut new_state = [0u8; STATE_SIZE];

    for slice in 0..SLICES {
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let old_idx = SLICESIZE * slice + COLUMNS * row + col;
                let new_slice = (slice + SHIFT_LANES_PARAM[col + COLUMNS * row]) % SLICES;
                let new_idx = SLICESIZE * (new_slice) + COLUMNS * row + col;
                new_state[new_idx] = state[old_idx];
            }
        }
    }
    
    state.copy_from_slice(&new_state[..]);
}

fn add_column_parity(state: &mut [Trit]) {
    let mut parity = [0u8; SLICES * COLUMNS];

    // First compute parity for each column
    for slice in 0..SLICES {
        for col in 0..COLUMNS {
            let mut col_sum = 0;
            for row in 0..ROWS {
                col_sum += state[SLICESIZE * slice + COLUMNS * row + col];
            }
            parity[COLUMNS * slice + col] = col_sum % 3;
        }
    }

    // Add parity
    for slice in 0..SLICES {
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let idx = SLICESIZE * slice + COLUMNS * row + col;
                let sum_to_add = parity[(col - 1 + 9) % 9 + COLUMNS * slice] + parity[(col + 1) % 9 + COLUMNS * ((slice + 1) % SLICES)];
                state[idx] = (state[idx] + sum_to_add) % 3;
            }
        }
    }
}

fn add_round_constant(state: &mut [Trit], round: usize) {
    for slice in 0..SLICES {
        for col in 0..COLUMNS {
            let idx = SLICESIZE * slice + col;
            state[idx] = (state[idx] + ROUND_CONSTANTS[round][slice * COLUMNS + col]) % 3;
        }
    }
}
