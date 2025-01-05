

pub struct FastSet {
    table: Vec<u64>,
    size: usize,
}

impl FastSet {
    #[inline(always)]
    fn hash(x: u64) -> u64 {
        // FxHash core mixing function
        const K: u64 = 0x517cc1b727220a95;
        (x.wrapping_mul(K)) >> 32
    }

    /*
    #[inline(always)]
    fn process_chunk(chunk: &mut[u64], key: u64) -> Option<bool> {
        // Pre-load the whole chunk into cache
        // By accessing first and last elements
        let _prefetch_head = chunk[0];
        let _prefetch_tail = chunk[chunk.len()-1];

        for val in chunk {
            if *val == key {
                return Some(false);
            } else if *val == 0 {
                *val = key;
                return Some(true);
            }
        }
        None // Continue searching
    }
    */
    
    /*
    // aligned new
    pub fn new(capacity: usize) -> Self {
        // Create aligned vector using Layout
        let layout = std::alloc::Layout::from_size_align(
            capacity * std::mem::size_of::<u64>(),
            64 // Align to cache line
        ).unwrap();

        let table = unsafe {
            let ptr = std::alloc::alloc_zeroed(layout);
            Vec::from_raw_parts(
                ptr as *mut u64,
                capacity,
                capacity
            )
        };

        Self {
            table,
            size: 0,
        }
    }

    fn drop(&mut self) {
        let layout = std::alloc::Layout::from_size_align(
            self.table.capacity() * std::mem::size_of::<u64>(),
            64
        ).unwrap();
        
        // Get the pointer before Vec is dropped
        let ptr = self.table.as_mut_ptr() as *mut u8;
        
        // Clear the Vec without dropping the memory
        unsafe {
            let _ = std::mem::ManuallyDrop::new(std::mem::replace(
                &mut self.table, 
                Vec::new()
            ));
            std::alloc::dealloc(ptr, layout);
        }
    }
    */

    // normal new
    pub fn new(capacity: usize) -> Self {
        let table = vec![0_u64; capacity];
        Self {
            table,
            size: 0,
        }
    }
    

    // simple version of insert
    #[inline]
    pub fn insert(&mut self, value: u64) -> bool {
        debug_assert!(value != 0, "Cannot insert 0, that’s the sentinel");
        debug_assert!(self.table.len().is_power_of_two()); // Help compiler optimize
       
        // let table_ptr = self.table.as_mut_ptr();
        // let mut start_idx = gxhash64(&value.to_le_bytes(),1234567890) as usize & (self.table.len() - 1); // GxHash
        let start_idx = Self::hash(value) as usize & (self.table.len() - 1); // FxHash, self.table.len() - 1
        let mut idx = start_idx;
        let mut collision_count = 0;
        let mask = self.table.len() - 1;
        /*
        // start_chunk is the position of head of first chunk (right side of start_idx)
        let start_chunk:usize;
        // end_chunk is the position of head of last chunk (left side of start_idx)
        let end_chunk:usize;

        if start_idx % 8 != 0 {
            start_chunk = ((start_idx >> 3) + 1) << 3;
            end_chunk = start_chunk - 8;
            if let Some(result) = Self::process_chunk(&mut self.table[start_idx..start_chunk], value) {
                if result {
                    self.size += 1;
                };
                return result;
            }
        } else {
            // start_idx is a multiple of 8
            start_chunk = start_idx;
            if start_idx > 0 {
                end_chunk = start_idx - 8;
            } else {
                end_chunk = 0;
            }
        }

        // process the right side of start_idx
        for chunk in self.table[start_chunk..].chunks_mut(8) {
            if let Some(result) = Self::process_chunk(chunk, value) {
                if result {
                    self.size += 1;
                };
                return result;
            }
        }
        // process the left side of start_idx
        if end_chunk > 0 {
            for chunk in self.table[0..end_chunk].chunks_mut(8) {
                if let Some(result) = Self::process_chunk(chunk, value) {
                    if result {
                        self.size += 1;
                    };
                    return result;
                }
            };
            // process the remains
            if let Some(result) = Self::process_chunk(&mut self.table[end_chunk..start_idx], value) {
                if result {
                    self.size += 1;
                };
                return result;
            }
        }

        // The table is full or we've tried everything
        panic!("Hash table is full");
    }*/

    // normal loop version
        loop {
            // unsafe version
            /*
            unsafe {
                let slot = *table_ptr.add(idx);
                if slot == 0 {
                    *table_ptr.add(idx) = value;
                    self.size += 1;
                    return true;
                } else if slot == value {
                    return false;
                }
            }
            */
            // safe version
            let slot = self.table[idx];
            if slot == 0 {
                self.table[idx] = value;
                self.size += 1;
                return true;
            } else if slot == value {
                return false;
            }  /* else {
                // increment collision counter each time we skip a filled slot
                collision_count += 1;
            }*/
            
            idx = (idx + 1) & mask;
            // println!("collision_count: {}", collision_count);
            if idx == start_idx {
                // The table is full or we've tried everything
                panic!("Hash table is full");
            }
        }
    }
        

    pub fn len(&self) -> usize {
        self.size
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