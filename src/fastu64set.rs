

pub struct FastSet {
    table: Vec<u64>,
    size: usize,
    collision_count: usize,
}

impl FastSet {
    #[inline(always)]
    fn hash(x: u64) -> u64 {
        // FxHash core mixing function
        const K: u64 = 0x517cc1b727220a95;
        (x.wrapping_mul(K)) >> 32
    }

    #[inline(always)]
    fn splitmix64(mut x: u64) -> u64 {
        x = x.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    #[inline(always)]
    fn xxhash64(input: u64) -> u64 {
        let mut hash = 0x9747b28c; // Prime number used as the seed.
        let mut temp = input;
    
        // Mix the bits of the input using a series of bitwise operations.
        temp ^= temp >> 31;
        temp = temp.wrapping_mul(0xbea225f9eb34556d);
        temp ^= temp >> 27;
        temp = temp.wrapping_mul(0x94d049bb133111eb);
        temp ^= temp >> 31;
    
        // Mix the bits of the hash and the input.
        hash ^= temp;
        hash = hash.rotate_left(27);
        hash = hash.wrapping_mul(0x9cb4bc65158fe28f);

        return hash;
    }
    // normal new
    pub fn new(capacity: usize) -> Self {
        let table = vec![0_u64; capacity];
        Self {
            table,
            size: 0,
            collision_count: 0,
        }
    }
    

    // simple version of insert
    #[inline]
    pub fn insert(&mut self, value: u64) -> bool {
        debug_assert!(value != 0, "Cannot insert 0, that’s the sentinel");
        debug_assert!(self.table.len().is_power_of_two()); // Help compiler optimize
       
        // let table_ptr = self.table.as_mut_ptr();
        // let mut start_idx = gxhash64(&value.to_le_bytes(),1234567890) as usize & (self.table.len() - 1); // GxHash
        let start_idx = Self::xxhash64(value) as usize & (self.table.len() - 1); // FxHash, self.table.len() - 1
        let mut idx = start_idx;
        let mask = self.table.len() - 1;
        // let mut i: usize= 1; // quadratic probing
        

    // normal loop version
        loop {
            // safe version
            let slot = self.table[idx];
            if slot == 0 {
                self.table[idx] = value;
                self.size += 1;
                return true;
            } else if slot == value {
                return false;
            } else {
                self.collision_count += 1;
            }
            
            idx = (idx + 1) & mask;
            //idx = (start_idx + i.wrapping_mul(i)) & mask;
            //i += 1;

            if idx == start_idx {
            //if i > self.table.len() {
                // The table is full or we've tried everything
                panic!("Hash table is full");
            }
        }
    }

    

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn collision_count(&self) -> usize {
        self.collision_count
    }

    #[inline]
    pub fn clear(&mut self) {
        // Fill table with 0
        // .fill() is usually efficient, but you can also use write_bytes if you prefer
        self.table.fill(0);
        self.size = 0;
    }

    pub fn extract(&self) -> Vec<u64> {
        // We know we'll never have more than size elements
        let mut out = Vec::with_capacity(self.size);
        for &slot in &self.table {
            if slot != 0 {
                out.push(slot);
            }
        }
        out
    }
}