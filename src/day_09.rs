use std::collections::VecDeque;

#[derive(Debug)]
struct DenseDiskPointer {
    index: usize,
    file_id: usize,
    is_file: bool,
    remaining_length: u8,
}

/// Calculate (offset + 0) * id + (offset + 1) * id + ... Writing this
/// out and simplifying ends up with the following formula.
fn block_checksum(offset: usize, length: usize, id: usize) -> usize {
    id * (length * offset + (length * (length - 1)) / 2)
}

pub fn part_a(input: &str) -> usize {
    let dense_disk_map: Vec<u8> = input.as_bytes().into_iter().map(|e| e - b'0').collect();
    assert!(dense_disk_map.len() > 0);

    let get_block_length = |idx| input.as_bytes()[idx] - b'0';

    let mut pointer_forward = DenseDiskPointer {
        index: 0,
        file_id: 0,
        is_file: true,
        remaining_length: get_block_length(0),
    };

    // Index to last file is the highest even index in the dense disk map.
    let last_file_idx = (dense_disk_map.len() - 1) - ((dense_disk_map.len() - 1) % 2);
    let last_file_id = last_file_idx / 2;
    let mut pointer_backward = DenseDiskPointer {
        index: last_file_idx,
        file_id: last_file_id,
        is_file: true,
        remaining_length: get_block_length(last_file_idx),
    };

    // Move forward though the dense map and consume either existing files, or
    // fill the free space by file blocks from the back.
    let mut result = 0;
    let mut block_position = 0;
    while pointer_forward.index <= pointer_backward.index {
        if pointer_forward.remaining_length == 0 {
            // If there's no remaining blocks, advance.
            let new_idx = pointer_forward.index + 1;
            pointer_forward = DenseDiskPointer {
                index: new_idx,
                file_id: pointer_forward.file_id + !pointer_forward.is_file as usize,
                is_file: !pointer_forward.is_file,
                remaining_length: get_block_length(new_idx),
            };
            log::debug!("Advanced fwd to {:?}", pointer_forward);
        } else if pointer_backward.remaining_length == 0 {
            // Jump to previous file.
            let new_idx = pointer_backward.index - 2;
            pointer_backward = DenseDiskPointer {
                index: new_idx,
                file_id: pointer_backward.file_id - 1,
                is_file: true,
                remaining_length: get_block_length(new_idx),
            };
            log::debug!("Advanced bkw to {:?}", pointer_backward);
        } else if pointer_forward.is_file {
            // If we are pointing to a file, consume it whole.
            log::debug!("Consuming fwd ({:?})", pointer_forward);

            // Fix remaining block count if both pointers are pointing at the same file.
            // NOTE: For some reason doing this when the forward pointer is advanced makes
            // things approximately 20% slower.
            if pointer_forward.index == pointer_backward.index {
                log::debug!(
                    "Correct fwd ({:?}) with bkw remaining length ({:?})",
                    pointer_forward,
                    pointer_backward
                );
                pointer_forward.remaining_length = pointer_backward.remaining_length;
            }

            result += block_checksum(
                block_position,
                pointer_forward.remaining_length.into(),
                pointer_forward.file_id.into(),
            );

            // Mark all blocks as consumed.
            block_position += pointer_forward.remaining_length as usize;
            pointer_forward.remaining_length = 0;
        } else {
            // Consume maximum amount of free blocks.
            let num_consumed_blocks = std::cmp::min(
                pointer_forward.remaining_length,
                pointer_backward.remaining_length,
            );

            // Consume one block of the forward pointer.
            log::debug!(
                "Consumed {} block(s) with fwd ({:?}) and bkw ({:?})",
                num_consumed_blocks,
                pointer_forward,
                pointer_backward
            );
            assert!(pointer_backward.is_file);
            result += block_checksum(
                block_position,
                num_consumed_blocks.into(),
                pointer_backward.file_id.into(),
            );

            // Advance pointers.
            block_position += num_consumed_blocks as usize;
            pointer_forward.remaining_length -= num_consumed_blocks;
            pointer_backward.remaining_length -= num_consumed_blocks;
        }
    }

    result
}

#[derive(Debug, Clone, Copy)]
struct FileBlock {
    offset: u32,
    id: u16,
    length: u8,
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
struct FreeBlock {
    offset: u32,
}

struct DiskMap {
    files: Vec<FileBlock>,
    free_space: Vec<VecDeque<FreeBlock>>,
}

impl DiskMap {
    const MAX_SPACE: usize = 9;

    fn new(num_blocks: usize) -> DiskMap {
        DiskMap {
            files: Vec::with_capacity((num_blocks + 1) / 2),
            free_space: std::iter::repeat_n(
                // Allocate enough to never require reallocation.
                VecDeque::with_capacity(num_blocks / 2),
                Self::MAX_SPACE + 1,
            )
            .collect(),
        }
    }
}

impl std::str::FromStr for DiskMap {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = DiskMap::new(s.len());

        s.as_bytes()
            .iter()
            .map(|length| length - b'0') // Convert char to length.
            .scan(0, |offset, length| {
                // Add offset to iterator.
                let cur_offset = *offset;
                *offset += length as u32;
                Some((cur_offset, length))
            })
            .enumerate() // Add block indices.
            .for_each(|(block_idx, (offset, length))| {
                match block_idx % 2 {
                    0 => {
                        // Even index => file
                        assert!(length > 0);
                        result.files.push(FileBlock {
                            id: (block_idx / 2) as u16,
                            length: length,
                            offset: offset,
                        });
                    }
                    1 => {
                        // Odd index => free space
                        match length {
                            0 => (), // Don't bother storing 0-length free blocks.
                            1.. => result.free_space[length as usize]
                                .push_back(FreeBlock { offset: offset }),
                        }
                    }
                    _ => unreachable!(),
                }
            });

        Ok(result)
    }
}

pub fn part_b(input: &str) -> usize {
    // Convert input to a list of file and free space blocks.
    let mut disk_map: DiskMap = input.parse().unwrap();
    let mut max_length_to_inspect = DiskMap::MAX_SPACE as u8;

    // Iterating backwards over the files, find the first location into which
    // all the file's blocks could be moved.
    for file in disk_map.files.iter_mut().rev() {
        // Don't do anything for files which we won't be able to move anyway.
        if file.length > max_length_to_inspect {
            continue;
        }

        // Find left-most free space that can hold file.
        let (free_length, free_offset) = (file.length..=max_length_to_inspect)
            .filter_map(|length| {
                // Left-most entry is always the first one in the list.
                let free_spaces = &disk_map.free_space[length as usize];
                match free_spaces.is_empty() {
                    true => None,
                    false => Some((length, free_spaces[0].offset)),
                }
            })
            .min_by_key(|(_, offset)| *offset)
            .unwrap_or((0, u32::MAX)); // Create a sentinel value.

        if free_offset > file.offset {
            // No free location with an offset less than the file's has been
            // found. Hence for any file of this length or larger, we'll never
            // find any suitable lower offset anymore. Therefore skip all those
            // files from now on.
            max_length_to_inspect = file.length - 1;
            log::debug!("Skipping all files of length > {}", max_length_to_inspect);

            // If all lengths are to be skipped, then stop iterating.
            if max_length_to_inspect == 0 {
                log::debug!("Stopping search at {:?}", file);
                break;
            }
        } else {
            // Move file to new location.
            let file_prev = *file;
            file.offset = free_offset;
            log::debug!("Moved {:?} to {:?}", file_prev, file);

            // Drop free block.
            disk_map.free_space[free_length as usize].pop_front();

            // Freed up blocks can't be reused by files in earlier blocks,
            // because that would move them backwards. But there might be free
            // space left in this block.
            if file.length < free_length {
                let new_length = free_length - file.length;
                let new_space = FreeBlock {
                    offset: free_offset + file.length as u32,
                };

                // Keep entries sorted. There shouldn't be offset duplicates.
                let free_spaces = &mut disk_map.free_space[new_length as usize];
                let insert_pos = free_spaces.binary_search(&new_space).unwrap_err();
                free_spaces.insert(insert_pos, new_space);
            }
        }
    }

    // Now just calculate the checksum.
    disk_map
        .files
        .iter()
        .map(|file| {
            let sum = block_checksum(file.offset as usize, file.length.into(), file.id.into());
            log::debug!("Adding {:?} => {}", file, sum);
            sum
        })
        .sum::<usize>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn example_a() {
        util::run_test(|| {
            let expected: usize = 1928;
            assert_eq!(
                crate::day_09::part_a(&util::read_resource("example_09.txt").unwrap()),
                expected
            );
        });
    }

    #[test]
    fn example_b() {
        util::run_test(|| {
            let expected: usize = 2858;
            assert_eq!(
                crate::day_09::part_b(&util::read_resource("example_09.txt").unwrap()),
                expected
            );
        });
    }
}
