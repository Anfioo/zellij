// 这是 sequence（以及可能其他）插件中同一文件的副本
// 目前除了 zellij-tile-utils crate 外，没有好的地方放置共享逻辑
// （我宁愿消除它），因此有重复。
use zellij_tile::prelude::*;

const MAX_UNDO_STACK_SIZE: usize = 100;

///TextInput 处理按键事件后返回的操作
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(Eq))]
pub enum InputAction {
    ///  继续编辑
    Continue,
    ///  用户按 Enter 提交
    Submit,
    ///  用户按 Esc 取消
    Cancel,
    ///  用户按 Tab 请求补全
    Complete,
    ///  输入未处理该按键
    NoAction,
}

///  一个可复用的文本输入组件，支持光标和标准编辑快捷键绑定
#[derive(Debug, Clone)]
pub struct TextInput {
    buffer: String,
    cursor_position: usize, //  字符位置（从 0 开始），不是字节位置
    undo_stack: Vec<(String, usize)>, //  (buffer, cursor) 快照
    redo_stack: Vec<(String, usize)>,
    last_edit_was_insert: bool, // 用于合并连续插入
}

impl TextInput {
    ///使用给定的初始文本创建新的 TextInput
    ///  光标位于文本末尾
    pub fn new(initial_text: String) -> Self {
        let cursor_position = initial_text.chars().count();
        Self {
            buffer: initial_text,
            cursor_position,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_edit_was_insert: false,
        }
    }

    ///创建一个空的 TextInput
    pub fn empty() -> Self {
        Self::new(String::new())
    }

    ///  获取当前文本
    pub fn get_text(&self) -> &str {
        &self.buffer
    }

    ///  获取光标位置（以字符为单位，不是字节）
    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    ///  检查输入是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    ///  获取 cursor_position 的简写
    #[allow(unused)]
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    ///  获取对底层缓冲区的可变访问以进行直接操作
    #[allow(unused)]
    pub fn get_text_mut(&mut self) -> &mut String {
        &mut self.buffer
    }

    ///  设置文本并将光标移到末尾
    #[allow(unused)]
    pub fn set_text(&mut self, text: String) {
        self.break_coalescing();
        self.save_undo_state();
        self.cursor_position = text.chars().count();
        self.buffer = text;
    }

    ///  设置光标位置（限制为文本长度）
    #[allow(unused)]
    pub fn set_cursor_position(&mut self, pos: usize) {
        let text_len = self.buffer.chars().count();
        self.cursor_position = pos.min(text_len);
    }

    ///  清除所有文本并重置光标
    pub fn clear(&mut self) {
        self.break_coalescing();
        self.save_undo_state();
        self.buffer.clear();
        self.cursor_position = 0;
    }

    ///  在当前光标位置插入一个字符
    pub fn insert_char(&mut self, c: char) {
        self.save_undo_state_unless_coalescing();
        //  将光标位置（字符索引）转换为字节索引
        let byte_index = self.char_index_to_byte_index(self.cursor_position);
        self.buffer.insert(byte_index, c);
        self.cursor_position += 1;
    }

    ///  删除光标前的字符（退格键）
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.break_coalescing();
            self.save_undo_state();
            self.cursor_position -= 1;
            let byte_index = self.char_index_to_byte_index(self.cursor_position);
            self.buffer.remove(byte_index);
        }
    }

    ///  删除光标位置的字符（删除键）
    pub fn delete(&mut self) {
        let len = self.buffer.chars().count();
        if self.cursor_position < len {
            self.break_coalescing();
            self.save_undo_state();
            let byte_index = self.char_index_to_byte_index(self.cursor_position);
            self.buffer.remove(byte_index);
        }
    }

    ///删除光标前的单词（Ctrl/Alt + 退格键）
    pub fn delete_word_backward(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        self.break_coalescing();
        self.save_undo_state();

        let old_position = self.cursor_position;
        self.move_word_left();
        let new_position = self.cursor_position;

        //  从新位置删除到旧位置
        let start_byte = self.char_index_to_byte_index(new_position);
        let end_byte = self.char_index_to_byte_index(old_position);
        self.buffer.drain(start_byte..end_byte);
    }

    ///删除光标后的单词（Ctrl/Alt + 删除键）
    pub fn delete_word_forward(&mut self) {
        let chars: Vec<char> = self.buffer.chars().collect();
        let len = chars.len();

        if self.cursor_position >= len {
            return;
        }

        self.break_coalescing();
        self.save_undo_state();

        let start_position = self.cursor_position;
        let mut end_position = start_position;

        //  跳过当前单词
        while end_position < len && !chars[end_position].is_whitespace() {
            end_position += 1;
        }

        //  跳过单词后的任何空白字符
        while end_position < len && chars[end_position].is_whitespace() {
            end_position += 1;
        }

        //  从起始位置删除到结束位置
        let start_byte = self.char_index_to_byte_index(start_position);
        let end_byte = self.char_index_to_byte_index(end_position);
        self.buffer.drain(start_byte..end_byte);
    }

    ///  将光标向左移动一个位置
    pub fn move_left(&mut self) {
        if self.cursor_position > 0 {
            self.break_coalescing();
            self.cursor_position -= 1;
        }
    }

    ///  将光标向右移动一个位置
    pub fn move_right(&mut self) {
        let len = self.buffer.chars().count();
        if self.cursor_position < len {
            self.break_coalescing();
            self.cursor_position += 1;
        }
    }

    ///将光标移动到文本开头（Ctrl-A / Home）
    pub fn move_to_start(&mut self) {
        self.break_coalescing();
        self.cursor_position = 0;
    }

    ///将光标移动到文本末尾（Ctrl-E / End）
    pub fn move_to_end(&mut self) {
        self.break_coalescing();
        self.cursor_position = self.buffer.chars().count();
    }

    ///将光标移动到上一个单词的开头（Ctrl/Alt + 左方向键）
    pub fn move_word_left(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        self.break_coalescing();

        let chars: Vec<char> = self.buffer.chars().collect();
        let mut pos = self.cursor_position;

        // 跳过紧邻左侧的任何空白字符
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // 跳过单词字符
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        self.cursor_position = pos;
    }

    ///将光标移动到下一个单词的开头（Ctrl/Alt + 右方向键）
    pub fn move_word_right(&mut self) {
        let chars: Vec<char> = self.buffer.chars().collect();
        let len = chars.len();

        if self.cursor_position >= len {
            return;
        }

        self.break_coalescing();

        let mut pos = self.cursor_position;

        //  跳过当前单词
        while pos < len && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // 跳过任何空白字符
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }

        self.cursor_position = pos;
    }

    ///  处理按键事件并返回适当的操作
    ///这是按键处理的主要入口点
    pub fn handle_key(&mut self, key: KeyWithModifier) -> InputAction {
        //  检查 Ctrl 修饰键
        if key.has_modifiers(&[KeyModifier::Ctrl]) {
            match key.bare_key {
                BareKey::Char('a') => {
                    self.move_to_start();
                    return InputAction::Continue;
                },
                BareKey::Char('e') => {
                    self.move_to_end();
                    return InputAction::Continue;
                },
                BareKey::Char('c') => {
                    //  Ctrl-C 清除提示
                    return InputAction::Cancel;
                },
                BareKey::Char('z') => {
                    // Ctrl-Z：撤销
                    self.undo();
                    return InputAction::Continue;
                },
                BareKey::Char('y') => {
                    //  Ctrl-Y：重做
                    self.redo();
                    return InputAction::Continue;
                },
                BareKey::Left => {
                    self.move_word_left();
                    return InputAction::Continue;
                },
                BareKey::Right => {
                    self.move_word_right();
                    return InputAction::Continue;
                },
                BareKey::Backspace => {
                    self.delete_word_backward();
                    return InputAction::Continue;
                },
                BareKey::Delete => {
                    self.delete_word_forward();
                    return InputAction::Continue;
                },
                _ => {},
            }
        }

        //  检查 Ctrl+Shift 修饰键（替代重做：Ctrl+Shift+Z）
        if key.has_modifiers(&[KeyModifier::Ctrl, KeyModifier::Shift]) {
            match key.bare_key {
                BareKey::Char('Z') => {
                    //  Ctrl-Shift-Z：重做（替代方式）
                    self.redo();
                    return InputAction::Continue;
                },
                _ => {},
            }
        }

        //  检查 Alt 修饰键
        if key.has_modifiers(&[KeyModifier::Alt]) {
            match key.bare_key {
                BareKey::Left => {
                    self.move_word_left();
                    return InputAction::Continue;
                },
                BareKey::Right => {
                    self.move_word_right();
                    return InputAction::Continue;
                },
                BareKey::Backspace => {
                    self.delete_word_backward();
                    return InputAction::Continue;
                },
                BareKey::Delete => {
                    self.delete_word_forward();
                    return InputAction::Continue;
                },
                _ => {},
            }
        }

        // 处理裸键（无修饰键）
        match key.bare_key {
            BareKey::Enter => InputAction::Submit,
            BareKey::Esc => InputAction::Cancel,
            BareKey::Tab => InputAction::Complete,
            BareKey::Backspace => {
                self.backspace();
                InputAction::Continue
            },
            BareKey::Delete => {
                self.delete();
                InputAction::Continue
            },
            BareKey::Left => {
                self.move_left();
                InputAction::Continue
            },
            BareKey::Right => {
                self.move_right();
                InputAction::Continue
            },
            BareKey::Home => {
                self.move_to_start();
                InputAction::Continue
            },
            BareKey::End => {
                self.move_to_end();
                InputAction::Continue
            },
            BareKey::Char(c) => {
                self.insert_char(c);
                InputAction::Continue
            },
            _ => InputAction::NoAction,
        }
    }

    ///  辅助：将字符索引转换为字节索引
    fn char_index_to_byte_index(&self, char_index: usize) -> usize {
        self.buffer
            .char_indices()
            .nth(char_index)
            .map(|(byte_idx, _)| byte_idx)
            .unwrap_or(self.buffer.len())
    }

    ///  在进行更改之前将当前状态保存到撤销栈
    fn save_undo_state(&mut self) {
        if self.undo_stack.len() >= MAX_UNDO_STACK_SIZE {
            self.undo_stack.remove(0);
        }
        self.undo_stack
            .push((self.buffer.clone(), self.cursor_position));
        self.redo_stack.clear();
    }

    ///  仅在不与前一次插入合并时保存状态
    fn save_undo_state_unless_coalescing(&mut self) {
        if !self.last_edit_was_insert {
            self.save_undo_state();
        }
        self.last_edit_was_insert = true;
    }

    ///  标记发生了非插入编辑（中断合并）
    fn break_coalescing(&mut self) {
        self.last_edit_was_insert = false;
    }

    ///  撤销上一次更改
    pub fn undo(&mut self) -> bool {
        if let Some((buffer, cursor)) = self.undo_stack.pop() {
            self.redo_stack
                .push((self.buffer.clone(), self.cursor_position));
            self.buffer = buffer;
            self.cursor_position = cursor;
            self.break_coalescing();
            true
        } else {
            false
        }
    }

    ///  重做上一次撤销的更改
    pub fn redo(&mut self) -> bool {
        if let Some((buffer, cursor)) = self.redo_stack.pop() {
            self.undo_stack
                .push((self.buffer.clone(), self.cursor_position));
            self.buffer = buffer;
            self.cursor_position = cursor;
            self.break_coalescing();
            true
        } else {
            false
        }
    }

    ///  检查撤销是否可用
    #[allow(unused)]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    ///  检查重做是否可用
    #[allow(unused)]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    #[allow(unused)]
    pub fn drain_text(&mut self) -> String {
        self.cursor_position = 0;
        self.buffer.drain(..).collect()
    }
}

// 运行方式：
// cargo test --lib --target x86_64-unknown-linux-gnu
//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_empty() {
        let input = TextInput::new("hello".to_string());
        assert_eq!(input.get_text(), "hello");
        assert_eq!(input.get_cursor_position(), 5);

        let empty = TextInput::empty();
        assert_eq!(empty.get_text(), "");
        assert_eq!(empty.get_cursor_position(), 0);
    }

    #[test]
    fn test_insert_char() {
        let mut input = TextInput::new("helo".to_string());
        input.cursor_position = 3; // 在 "hel" 之后的位置
        input.insert_char('l');
        assert_eq!(input.get_text(), "hello");
        assert_eq!(input.get_cursor_position(), 4);
    }

    #[test]
    fn test_backspace() {
        let mut input = TextInput::new("hello".to_string());
        input.backspace();
        assert_eq!(input.get_text(), "hell");
        assert_eq!(input.get_cursor_position(), 4);

        //  在开头按退格键不执行任何操作
        input.cursor_position = 0;
        input.backspace();
        assert_eq!(input.get_text(), "hell");
        assert_eq!(input.get_cursor_position(), 0);
    }

    #[test]
    fn test_delete() {
        let mut input = TextInput::new("hello".to_string());
        input.cursor_position = 0;
        input.delete();
        assert_eq!(input.get_text(), "ello");
        assert_eq!(input.get_cursor_position(), 0);

        // 在末尾删除不执行任何操作
        input.move_to_end();
        input.delete();
        assert_eq!(input.get_text(), "ello");
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = TextInput::new("hello".to_string());
        assert_eq!(input.get_cursor_position(), 5);

        input.move_left();
        assert_eq!(input.get_cursor_position(), 4);

        input.move_right();
        assert_eq!(input.get_cursor_position(), 5);

        input.move_to_start();
        assert_eq!(input.get_cursor_position(), 0);

        input.move_to_end();
        assert_eq!(input.get_cursor_position(), 5);
    }

    #[test]
    fn test_unicode_support() {
        let mut input = TextInput::new("hello 🦀 world".to_string());
        assert_eq!(input.get_cursor_position(), 13); //  13 个字符

        input.cursor_position = 6; //  在 "hello " 之后
        input.insert_char('🐱');
        assert_eq!(input.get_text(), "hello 🐱🦀 world");
    }

    #[test]
    fn test_word_jump_right() {
        let mut input = TextInput::new("hello world foo bar".to_string());
        input.cursor_position = 0;

        // 从开头跳转到 "world"
        input.move_word_right();
        assert_eq!(input.get_cursor_position(), 6); //  在 "hello " 之后

        // 跳转到 "foo"
        input.move_word_right();
        assert_eq!(input.get_cursor_position(), 12); //  在 "world " 之后

        // 跳转到 "bar"
        input.move_word_right();
        assert_eq!(input.get_cursor_position(), 16); //  在 "foo " 之后

        // 跳转到末尾
        input.move_word_right();
        assert_eq!(input.get_cursor_position(), 19); //  在末尾
    }

    #[test]
    fn test_word_jump_left() {
        let mut input = TextInput::new("hello world foo bar".to_string());
        input.move_to_end();
        assert_eq!(input.get_cursor_position(), 19);

        // 跳回到 "bar"
        input.move_word_left();
        assert_eq!(input.get_cursor_position(), 16); // "bar" 的开头

        // 跳回到 "foo"
        input.move_word_left();
        assert_eq!(input.get_cursor_position(), 12); // "foo" 的开头

        // 跳回到 "world"
        input.move_word_left();
        assert_eq!(input.get_cursor_position(), 6); // "world" 的开头

        // 跳回到 "hello"
        input.move_word_left();
        assert_eq!(input.get_cursor_position(), 0); // "hello" 的开头
    }

    #[test]
    fn test_word_jump_with_multiple_spaces() {
        let mut input = TextInput::new("hello   world".to_string());
        input.cursor_position = 0;

        // 跳过多个空格
        input.move_word_right();
        assert_eq!(input.get_cursor_position(), 8); //  在 "hello   " 之后，在 "world" 开头

        // 向后跳转应跳过空格
        input.move_word_left();
        assert_eq!(input.get_cursor_position(), 0); //  回到 "hello" 的开头
    }

    #[test]
    fn test_word_jump_boundaries() {
        let mut input = TextInput::new("test".to_string());

        //  在开头 - 向左移单词不执行任何操作
        input.cursor_position = 0;
        input.move_word_left();
        assert_eq!(input.get_cursor_position(), 0);

        //  在末尾 - 向右移单词不执行任何操作
        input.move_to_end();
        let end_pos = input.get_cursor_position();
        input.move_word_right();
        assert_eq!(input.get_cursor_position(), end_pos);
    }

    #[test]
    fn test_up_down_arrows() {
        let mut input = TextInput::new("hello world".to_string());

        // 从中间开始
        input.cursor_position = 5;
        assert_eq!(input.get_cursor_position(), 5);

        // 上方向键应跳到开头
        input.move_to_start();
        assert_eq!(input.get_cursor_position(), 0);

        // 移回中间
        input.cursor_position = 5;

        // 下方向键应跳到末尾
        input.move_to_end();
        assert_eq!(input.get_cursor_position(), 11);
    }

    #[test]
    fn test_delete_word_backward() {
        let mut input = TextInput::new("hello world foo".to_string());

        // 从末尾删除 "foo"
        input.move_to_end();
        input.delete_word_backward();
        assert_eq!(input.get_text(), "hello world ");
        assert_eq!(input.get_cursor_position(), 12);

        // 删除 "world "
        input.delete_word_backward();
        assert_eq!(input.get_text(), "hello ");
        assert_eq!(input.get_cursor_position(), 6);

        // 删除 "hello "
        input.delete_word_backward();
        assert_eq!(input.get_text(), "");
        assert_eq!(input.get_cursor_position(), 0);

        // 在空缓冲区上删除不执行任何操作
        input.delete_word_backward();
        assert_eq!(input.get_text(), "");
        assert_eq!(input.get_cursor_position(), 0);
    }

    #[test]
    fn test_delete_word_backward_middle() {
        let mut input = TextInput::new("hello world foo".to_string());

        // 在 "world" 中间的位置
        input.cursor_position = 8; //  在 "hello wo" 之后
        input.delete_word_backward();
        assert_eq!(input.get_text(), "hello rld foo");
        assert_eq!(input.get_cursor_position(), 6); //  在 "hello " 之后
    }

    #[test]
    fn test_delete_word_forward() {
        let mut input = TextInput::new("hello world foo".to_string());

        // 从开头删除 "hello "
        input.cursor_position = 0;
        input.delete_word_forward();
        assert_eq!(input.get_text(), "world foo");
        assert_eq!(input.get_cursor_position(), 0);

        // 删除 "world "
        input.delete_word_forward();
        assert_eq!(input.get_text(), "foo");
        assert_eq!(input.get_cursor_position(), 0);

        // 删除 "foo"
        input.delete_word_forward();
        assert_eq!(input.get_text(), "");
        assert_eq!(input.get_cursor_position(), 0);

        // 在空缓冲区上删除不执行任何操作
        input.delete_word_forward();
        assert_eq!(input.get_text(), "");
        assert_eq!(input.get_cursor_position(), 0);
    }

    #[test]
    fn test_delete_word_forward_middle() {
        let mut input = TextInput::new("hello world foo".to_string());

        // 在 "world" 中间的位置
        input.cursor_position = 8; //  在 "hello wo" 之后
        input.delete_word_forward();
        assert_eq!(input.get_text(), "hello wofoo");
        assert_eq!(input.get_cursor_position(), 8); // 相同位置，文本被向前删除
    }

    #[test]
    fn test_delete_word_with_multiple_spaces() {
        let mut input = TextInput::new("hello   world".to_string());

        // 向前删除包括尾随空格
        input.cursor_position = 0;
        input.delete_word_forward();
        assert_eq!(input.get_text(), "world");
        assert_eq!(input.get_cursor_position(), 0);
    }

    #[test]
    fn test_undo_redo_basic() {
        let mut input = TextInput::empty();

        // 输入 "hello"
        input.insert_char('h');
        input.insert_char('e');
        input.insert_char('l');
        input.insert_char('l');
        input.insert_char('o');
        assert_eq!(input.get_text(), "hello");

        // 撤销应移除所有字符（合并为一个撤销条目）
        assert!(input.can_undo());
        assert!(input.undo());
        assert_eq!(input.get_text(), "");
        assert_eq!(input.get_cursor_position(), 0);

        // 重做应恢复 "hello"
        assert!(input.can_redo());
        assert!(input.redo());
        assert_eq!(input.get_text(), "hello");
        assert_eq!(input.get_cursor_position(), 5);
    }

    #[test]
    fn test_undo_coalescing_breaks_on_cursor_move() {
        let mut input = TextInput::empty();

        // 输入 "he"
        input.insert_char('h');
        input.insert_char('e');

        // 将光标移动到开头（中断合并）
        input.move_to_start();

        // 输入 "llo"
        input.insert_char('l');
        input.insert_char('l');
        input.insert_char('o');

        assert_eq!(input.get_text(), "llohe");

        // 第一次撤销移除 "llo"（第二个合并组）
        input.undo();
        assert_eq!(input.get_text(), "he");

        // 第二次撤销移除 "he"（第一个合并组）
        input.undo();
        assert_eq!(input.get_text(), "");
    }

    #[test]
    fn test_undo_backspace() {
        let mut input = TextInput::new("hello".to_string());

        //  按一次退格键
        input.backspace();
        assert_eq!(input.get_text(), "hell");

        // 撤销应恢复 "hello"
        input.undo();
        assert_eq!(input.get_text(), "hello");
        assert_eq!(input.get_cursor_position(), 5);
    }

    #[test]
    fn test_undo_delete() {
        let mut input = TextInput::new("hello".to_string());
        input.cursor_position = 0;

        // 删除第一个字符
        input.delete();
        assert_eq!(input.get_text(), "ello");

        // 撤销应恢复 "hello"
        input.undo();
        assert_eq!(input.get_text(), "hello");
        assert_eq!(input.get_cursor_position(), 0);
    }

    #[test]
    fn test_undo_word_delete() {
        let mut input = TextInput::new("hello world".to_string());

        // 向后删除 "world"
        input.delete_word_backward();
        assert_eq!(input.get_text(), "hello ");

        // 撤销应恢复 "hello world"
        input.undo();
        assert_eq!(input.get_text(), "hello world");
        assert_eq!(input.get_cursor_position(), 11);
    }

    #[test]
    fn test_undo_clear() {
        let mut input = TextInput::new("hello world".to_string());

        //  清除缓冲区
        input.clear();
        assert_eq!(input.get_text(), "");

        // 撤销应恢复文本
        input.undo();
        assert_eq!(input.get_text(), "hello world");
    }

    #[test]
    fn test_undo_set_text() {
        let mut input = TextInput::new("hello".to_string());

        // 替换为新文本
        input.set_text("goodbye".to_string());
        assert_eq!(input.get_text(), "goodbye");

        // 撤销应恢复 "hello"
        input.undo();
        assert_eq!(input.get_text(), "hello");
    }

    #[test]
    fn test_redo_clears_on_new_edit() {
        let mut input = TextInput::empty();

        // 输入 "hello"
        input.insert_char('h');
        input.insert_char('e');
        input.insert_char('l');
        input.insert_char('l');
        input.insert_char('o');

        // 撤销
        input.undo();
        assert_eq!(input.get_text(), "");
        assert!(input.can_redo());

        // 进行新编辑（应清除重做栈）
        input.insert_char('x');
        assert!(!input.can_redo());
    }

    #[test]
    fn test_multiple_undo_redo() {
        let mut input = TextInput::empty();

        // 第一次编辑：输入 "hello"
        for c in "hello".chars() {
            input.insert_char(c);
        }

        //  中断合并
        input.move_left();

        // 第二次编辑：输入 "world"
        for c in "world".chars() {
            input.insert_char(c);
        }

        assert_eq!(input.get_text(), "hellworldo");

        // 撤销 "world"
        input.undo();
        assert_eq!(input.get_text(), "hello");

        // 撤销 "hello"
        input.undo();
        assert_eq!(input.get_text(), "");

        // 重做 "hello"
        input.redo();
        assert_eq!(input.get_text(), "hello");

        // 重做 "world"
        input.redo();
        assert_eq!(input.get_text(), "hellworldo");
    }

    #[test]
    fn test_undo_stack_limit() {
        let mut input = TextInput::empty();

        // 执行 102 次独立编辑（每次都中断合并）
        for _i in 0..102 {
            input.backspace(); //  中断合并
            input.insert_char('x');
        }

        // 最多应有 100 个撤销条目
        let mut undo_count = 0;
        while input.undo() {
            undo_count += 1;
        }

        // 我们应该有 100 个撤销条目（栈限制）
        // 加上最后一次合并中断的最终状态更改
        assert!(
            undo_count <= 100,
            "Undo count should be at most 100, got {}",
            undo_count
        );
    }

    #[test]
    fn test_undo_redo_empty_stack() {
        let mut input = TextInput::empty();

        // 在空栈上撤销应返回 false
        assert!(!input.can_undo());
        assert!(!input.undo());

        // 在空栈上重做应返回 false
        assert!(!input.can_redo());
        assert!(!input.redo());
    }

    #[test]
    fn test_undo_restores_cursor_position() {
        let mut input = TextInput::new("hello world".to_string());

        // 将光标移动到位置 5（在 "world" 之前）
        input.cursor_position = 5;

        // 插入一个空格
        input.insert_char(' ');
        assert_eq!(input.get_text(), "hello  world");
        assert_eq!(input.get_cursor_position(), 6);

        // 撤销应同时恢复文本和光标位置
        input.undo();
        assert_eq!(input.get_text(), "hello world");
        assert_eq!(input.get_cursor_position(), 5);
    }

    #[test]
    fn test_coalescing_consecutive_inserts() {
        let mut input = TextInput::empty();

        // 输入多个字符
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');

        assert_eq!(input.get_text(), "abc");

        // 单次撤销应移除全部三个（它们已合并）
        input.undo();
        assert_eq!(input.get_text(), "");

        // 没有更多可用的撤销
        assert!(!input.can_undo());
    }

    #[test]
    fn test_backspace_breaks_coalescing() {
        let mut input = TextInput::empty();

        // 输入 "ab"
        input.insert_char('a');
        input.insert_char('b');

        //  退格键
        input.backspace();
        assert_eq!(input.get_text(), "a");

        // 输入 "c"
        input.insert_char('c');
        assert_eq!(input.get_text(), "ac");

        // 撤销应仅移除 "c"
        input.undo();
        assert_eq!(input.get_text(), "a");

        // 撤销应移除退格操作
        input.undo();
        assert_eq!(input.get_text(), "ab");

        //  撤销应移除 "ab"
        input.undo();
        assert_eq!(input.get_text(), "");
    }
}
