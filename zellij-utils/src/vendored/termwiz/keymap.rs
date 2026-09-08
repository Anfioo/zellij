//! 用于保存键映射条目的数据结构
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct Node<Value: Debug> {
    label: u8,
    children: Vec<Node<Value>>,
    value: Option<Value>,
}

impl<Value: Debug> Node<Value> {
    fn new(label: u8) -> Self {
        Self {
            label,
            children: Vec::new(),
            value: None,
        }
    }

    fn insert(&mut self, key: &[u8], value: Value) {
        if key.is_empty() {
            // 已到达叶子节点
            self.value = Some(value);
            return;
        }
        match self
            .children
            .binary_search_by(|node| node.label.cmp(&key[0]))
        {
            Ok(idx) => {
                self.children[idx].insert(&key[1..], value);
            },
            Err(idx) => {
                self.children.insert(idx, Node::new(key[0]));
                self.children[idx].insert(&key[1..], value);
            },
        }
    }

    fn lookup(&self, key: &[u8], depth: usize, maybe_more: bool) -> NodeFind<&Value> {
        if key.is_empty() {
            // 已匹配到输入键的最大范围。
            if self.children.is_empty() {
                match self.value.as_ref() {
                    Some(value) => {
                        // 对整个输入的明确匹配
                        return NodeFind::Exact(depth, value);
                    },
                    None => panic!("Node has no children and no value!?"),
                }
            }
            return match self.value.as_ref() {
                Some(value) => {
                    if maybe_more {
                        NodeFind::AmbiguousMatch(depth, value)
                    } else {
                        NodeFind::Exact(depth, value)
                    }
                },
                None => NodeFind::AmbiguousBackTrack,
            };
        }

        match self
            .children
            .binary_search_by(|node| node.label.cmp(&key[0]))
        {
            Ok(idx) => {
                match self.children[idx].lookup(&key[1..], depth + 1, maybe_more) {
                    NodeFind::AmbiguousBackTrack => {
                        // 子节点没有精确匹配，因此检查我们是否有
                        match self.value.as_ref() {
                            Some(value) => {
                                // 我们有！如果期望更多数据，则返回 AmbiguousMatch，
                                // 否则将其视为 Exact 匹配
                                //
                                // 示例请参见本文件中的
                                // "lookup_with_multiple_ambiguous_matches_" 测试用例
                                if maybe_more {
                                    NodeFind::AmbiguousMatch(depth, value)
                                } else {
                                    NodeFind::Exact(depth, value)
                                }
                            },
                            None => NodeFind::AmbiguousBackTrack,
                        }
                    },
                    result => result,
                }
            },
            Err(_) => {
                if depth == 0 {
                    NodeFind::None
                } else {
                    match self.value.as_ref() {
                        Some(value) => NodeFind::Exact(depth, value),
                        None => NodeFind::AmbiguousBackTrack,
                    }
                }
            },
        }
    }
}

/// 内部查找处置
enum NodeFind<Value> {
    /// 无可能的匹配
    None,
    /// 找到精确匹配。（匹配长度，值）
    Exact(usize, Value),
    /// 在键的完整范围内未找到精确匹配，
    /// 因此请求上层回溯以查找部分匹配。
    AmbiguousBackTrack,
    /// 回溯后找到前缀匹配，但我们知道给定更多数据后
    /// 可能存在更具体的匹配。（匹配长度，值）。
    AmbiguousMatch(usize, Value),
}

/// 保存查找操作的结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found<Value> {
    /// 确定没有可能的匹配
    None,
    /// 我们找到了明确匹配。
    /// 数据为（匹配长度，值）
    Exact(usize, Value),
    /// 我们找到了匹配，但还存在其他可能的更长匹配。
    /// 理想情况下我们会积累更多数据以确定。
    /// 数据为（最短匹配长度，值）
    Ambiguous(usize, Value),
    /// 如果有更多数据，我们可能会匹配到某些内容
    NeedData,
}

/// `KeyMap` 结构体用于保存 unix 终端程序生成的文本序列。
/// 这些序列具有重叠/歧义的含义，需要更多数据才能正确解释。
/// `lookup` 操作返回一个描述匹配置信度的枚举，而非简单的 map 查找。
#[derive(Debug, Clone)]
pub struct KeyMap<Value: Debug + Clone> {
    root: Node<Value>,
}

impl<Value: Debug + Clone> Default for KeyMap<Value> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Value: Debug + Clone> KeyMap<Value> {
    pub fn new() -> Self {
        Self { root: Node::new(0) }
    }

    /// 向键映射中插入一个值
    pub fn insert<K: AsRef<[u8]>>(&mut self, key: K, value: Value) {
        self.root.insert(key.as_ref(), value)
    }

    /// 对 `key` 执行查找。
    /// `key` 可以是由字节序列组成的字符串。
    /// `lookup` 操作会考虑 `key` 的前缀并搜索匹配。
    ///
    /// 如果返回 `Found::None`，则 key 的前缀没有匹配的键映射条目。
    ///
    /// 如果返回 `Found::Exact`，则返回值告知调用者匹配的键长度；
    /// 键的其余部分未被考虑，应在后续查找操作中再次考虑。
    ///
    /// 如果返回 `Found::Ambiguous`，则键匹配了一个有效条目（作为值返回），
    /// 但如果有更多数据可用，至少还有一个其他条目可能匹配。如果调用者知道
    /// 立即没有更多数据可用，则将此结果视为等同于 `Found::Exact` 可能是有效的。
    /// 此变体的预期用途是处理序列跨越缓冲区边界的情况（例如：固定大小缓冲区
    /// 接收到部分序列，其余部分在下一次读取时立即可用），而不会误解读取的数据。
    ///
    /// 如果返回 `Found::NeedData`，则表示 `key` 太短，无法确定匹配。目的类似于
    /// `Found::Ambiguous` 情况；如果调用者知道没有更多数据可用，可将其视为
    /// `Found::None`，否则最好从流中读取更多数据并用更长的输入重试。
    pub fn lookup<S: AsRef<[u8]>>(&self, key: S, maybe_more: bool) -> Found<Value> {
        match self.root.lookup(key.as_ref(), 0, maybe_more) {
            NodeFind::None => Found::None,
            NodeFind::AmbiguousBackTrack => Found::NeedData,
            NodeFind::Exact(depth, value) => Found::Exact(depth, value.clone()),
            NodeFind::AmbiguousMatch(depth, value) => Found::Ambiguous(depth, value.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const NO_MORE: bool = false;
    const MAYBE_MORE: bool = true;

    #[test]
    fn lookup_empty() {
        let km: KeyMap<bool> = KeyMap::new();
        assert_eq!(km.lookup("boo", true), Found::None);
    }

    #[test]
    fn lookup() {
        let mut km = KeyMap::new();
        km.insert("boa", true);
        km.insert("boo", true);
        km.insert("boom", false);
        assert_eq!(km.lookup("b", MAYBE_MORE), Found::NeedData);
        assert_eq!(km.lookup("bo", MAYBE_MORE), Found::NeedData);
        assert_eq!(km.lookup("boa", MAYBE_MORE), Found::Exact(3, true),);
        assert_eq!(km.lookup("boo", MAYBE_MORE), Found::Ambiguous(3, true),);
        assert_eq!(km.lookup("boom", MAYBE_MORE), Found::Exact(4, false),);
        assert_eq!(km.lookup("boom!", MAYBE_MORE), Found::Exact(4, false),);
    }

    #[test]
    fn lookup_with_multiple_ambiguous_matches_without_additional_input() {
        let mut km = KeyMap::new();
        km.insert("boa", false);
        km.insert("boo", false);
        km.insert("boom", true);
        km.insert("boom!!", false);
        assert_eq!(km.lookup("boom!", NO_MORE), Found::Exact(4, true));
    }

    #[test]
    fn lookup_with_multiple_ambiguous_matches_with_potential_additional_input() {
        let mut km = KeyMap::new();
        km.insert("boa", false);
        km.insert("boo", false);
        km.insert("boom", true);
        km.insert("boom!!", false);
        assert_eq!(km.lookup("boom!", MAYBE_MORE), Found::Ambiguous(4, true));
    }

    #[test]
    fn sequence() {
        let mut km = KeyMap::new();
        km.insert("\x03", true);
        km.insert("\x27", true);
        km.insert("\x03XYZ", true);
        assert_eq!(km.lookup("\x03", MAYBE_MORE), Found::Ambiguous(1, true),);
        assert_eq!(km.lookup("\x03foo", MAYBE_MORE), Found::Exact(1, true),);
        assert_eq!(km.lookup("\x03X", MAYBE_MORE), Found::Ambiguous(1, true),);
    }
}
