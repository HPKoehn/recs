
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct GenerationalIndex {
    index: usize,
    generation: u64,
}

impl GenerationalIndex {
    pub fn index(&self) -> usize {
        return self.index;
    }
}

struct AllocatorEntry {
    is_live: bool,
    generation: u64,
}

pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}

impl GenerationalIndexAllocator {
    pub fn new() -> GenerationalIndexAllocator {
        GenerationalIndexAllocator {
            entries: Vec::new(),
            free: Vec::new()
        }
    }

    pub fn allocate(&mut self) -> GenerationalIndex {
        // check if we can reuse and unused entry
        if !self.free.is_empty() {
            // we unwrap as we just checked for content
            let potential_index = self.free.pop().unwrap();

            // check if the index is actually free, else go to new allocation
            if let Some(allocator_entry) = self.entries.get_mut(potential_index) {
                if !allocator_entry.is_live {
                    // adjust allocator entry
                    allocator_entry.is_live = true;
                    allocator_entry.generation += 1;

                    return GenerationalIndex {
                        index: potential_index,
                        generation: allocator_entry.generation
                    };
                }
            }
        }

        // allocate a completly new index
        let index = self.entries.len();
        let generation = 0;

        self.entries.push(AllocatorEntry{
            is_live: true,
            generation: generation
        });

        return GenerationalIndex {
            index,
            generation
        };
    }

    pub fn deallocate(&mut self, index: GenerationalIndex) -> bool {
        let allocator_entry_option = self.entries.get_mut(index.index());

        match allocator_entry_option {
            Some(allocator_entry) => {
                if !allocator_entry.is_live {
                    return false;
                }

                if allocator_entry.generation != index.generation {
                    return false;
                }

                allocator_entry.is_live = false;
                self.free.push(index.index());
                return true;
            },
            None => {
                return false;
            }
        }
    }

    pub fn is_live(&self, index: GenerationalIndex) -> bool {
        if let Some(allocator_entry) = self.entries.get(index.index()) {
            allocator_entry.generation == index.generation && allocator_entry.is_live
        } else {
            false
        }
    }
}

struct ArrayEntry<T> {
    value: T,
    generation: u64,
}

pub struct GenerationalIndexArray<T>(Vec<Option<ArrayEntry<T>>>);

impl<T> GenerationalIndexArray<T> {
    pub fn new() -> GenerationalIndexArray<T> {
        GenerationalIndexArray {
            0: Vec::new()
        }
    }

    pub fn set(&mut self, index: GenerationalIndex, value: T) {
        let inx = index.index();
        // extend vector if too short
        while self.0.len() <= inx + 1 {
            self.0.push(None);
        }
        self.0[inx] = Some(ArrayEntry {
            value,
            generation: index.generation
        });
    }

    pub fn get(&self, index: GenerationalIndex) -> Option<&T> {
        if self.0.len() <= index.index {
            return None;
        }
        match &self.0[index.index()] {
            None => None,
            Some(entry) => {
                if index.generation == entry.generation {
                    Some(&entry.value)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_mut(&mut self, index: GenerationalIndex) -> Option<&mut T> {
        if self.0.len() <= index.index() {
            return None;
        }
        match &mut self.0[index.index()] {
            None => None,
            Some(entry) => {
                if index.generation == entry.generation {
                    Some(&mut entry.value)
                } else {
                    None
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_three_entries() {
        let mut allocator = GenerationalIndexAllocator::new();
        let index_1 = allocator.allocate();
        let index_2 = allocator.allocate();
        let index_3 = allocator.allocate();

        assert_eq!(index_1.index(), 0);
        assert_eq!(index_2.index(), 1);
        assert_eq!(index_3.index(), 2);

        assert_eq!(index_1.generation, 0);
        assert_eq!(index_2.generation, 0);
        assert_eq!(index_3.generation, 0);

        assert_eq!(allocator.entries.len(), 3);
        for allocator_entry in allocator.entries {
            assert!(allocator_entry.is_live);
        }
    }

    #[test]
    fn test_reallocate() {
        let mut allocator = GenerationalIndexAllocator::new();
        let index = allocator.allocate();
        allocator.deallocate(index);
        let new_index = allocator.allocate();

        assert_ne!(index, new_index);
        assert_eq!(index.index(), new_index.index());

        assert_eq!(new_index.generation, 1);
    }

    #[test]
    fn test_reallocation_interior() {
        let mut allocator = GenerationalIndexAllocator::new();
        let old_index_1 = allocator.allocate();
        let _old_index_2 = allocator.allocate();

        assert_eq!(allocator.entries.len(), 2);
        assert_eq!(allocator.free.len(), 0);
        assert!(allocator.deallocate(old_index_1));
        assert_eq!(allocator.entries.len(), 2);
        assert_eq!(allocator.free.len(), 1);

        assert!(!allocator.entries[0].is_live);
        assert!(allocator.entries[1].is_live);
        assert_eq!(allocator.free[0], 0);
    }

    #[test]
    fn test_reallocation_generations() {
        let mut allocator = GenerationalIndexAllocator::new();
        let max_generations = 10;

        for i in 0..max_generations {
            let index = allocator.allocate();
            assert_eq!(index.generation, i);
            assert!(allocator.deallocate(index));
        }
    }
}
