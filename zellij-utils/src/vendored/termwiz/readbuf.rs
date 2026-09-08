/// 这是一个简单小巧的读取缓冲区，其内容始终以连续切片的形式可用。
#[derive(Debug)]
pub struct ReadBuffer {
    storage: Vec<u8>,
}

impl ReadBuffer {
    pub fn new() -> Self {
        Self {
            storage: Vec::with_capacity(16),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.storage.as_slice()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// 将 `len` 个字节标记为已消费，丢弃它们并将缓冲区内容前移，
    /// 使剩余字节位于缓冲区前端。
    pub fn advance(&mut self, len: usize) {
        let remain = self.storage.len() - len;
        self.storage.rotate_left(len);
        self.storage.truncate(remain);
    }

    /// 将切片内容追加到读取缓冲区
    pub fn extend_with(&mut self, slice: &[u8]) {
        self.storage.extend_from_slice(slice);
    }

    /// 从 `offset` 开始搜索 `needle`。找到则返回其在缓冲区中的偏移量，否则返回 None。
    pub fn find_subsequence(&self, offset: usize, needle: &[u8]) -> Option<usize> {
        self.storage[offset..]
            .windows(needle.len())
            .position(|w| w == needle)
            .map(|pos| pos + offset)
    }
}
