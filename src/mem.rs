pub fn align_down(address: usize, granularity: usize) -> usize {
  address & (!(granularity-1))
}

pub fn align_up(address: usize, granularity: usize) -> usize {
  (address+granularity) & (!(granularity-1))
}

pub fn physical_from_kernel(kernel: usize) -> usize {
  kernel & (0x0000008000000000-1)
}
