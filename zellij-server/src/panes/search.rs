use super::Selection;
use crate::panes::terminal_character::TerminalCharacter;
use crate::panes::{Grid, Row};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt::Debug;
use zellij_utils::input::actions::SearchDirection;
use zellij_utils::position::Position;

// 如果字符既不是字母数字也不是下划线，我们是否将其视为单词边界
fn is_word_boundary(x: &Option<char>) -> bool {
    x.map_or(true, |c| !c.is_ascii_alphanumeric() && c != '_')
}

#[derive(Debug)]
enum SearchSource<'a> {
    Main(&'a Row),
    Tail(&'a Row),
}

impl<'a> SearchSource<'a> {
    /// 如果找到新的源则返回 true，否则返回 false（到达尾部末尾）。
    /// 如果我们在行的中间，不会有任何改变。
    /// 只有当我们必须切换到新行时，源才会更新自身，
    /// 以及相应的索引。
    fn get_next_source(
        &mut self,
        ridx: &mut usize,
        hidx: &mut usize,
        tailit: &mut std::slice::Iter<&'a Row>,
        start: &Option<Position>,
    ) -> bool {
        match self {
            SearchSource::Main(row) => {
                // 如果我们在主行的末尾，需要开始查看尾部
                if hidx >= &mut row.columns.len() {
                    let curr_tail = tailit.next();
                    // 如果我们在末尾并找到了部分匹配，必须将搜索扩展到下一行
                    if let Some(curr_tail) = start.and(curr_tail) {
                        *ridx += 1; // 向下一行
                        *hidx = 0; // 并从新行的开头开始
                        *self = SearchSource::Tail(curr_tail);
                    } else {
                        return false; // 我们到达了尾部的末尾
                    }
                }
            },
            SearchSource::Tail(tail) => {
                if hidx >= &mut tail.columns.len() {
                    // 如果我们仍在搜索（尚未遇到不匹配）并且还有更多尾部可查
                    // 就继续下一行
                    if let Some(curr_tail) = tailit.next() {
                        *ridx += 1; // 向下一行
                        *hidx = 0; // 并从新行的开头开始
                        *self = SearchSource::Tail(curr_tail);
                    } else {
                        return false; // 我们到达了尾部的末尾
                    }
                }
            },
        }
        // 我们找到了新的源，或者我们在行的中间，因此无需更改任何内容
        true
    }

    // 获取 hidx 处的字符，如果存在，还获取下一个字符
    fn get_next_two_chars(&self, hidx: usize, whole_word_search: bool) -> (char, Option<char>) {
        // 获取当前干草堆字符
        let haystack_char = match self {
            SearchSource::Main(row) => row.columns[hidx].character,
            SearchSource::Tail(tail) => tail.columns[hidx].character,
        };

        // 获取下一个干草堆字符（仅对全词搜索相关）
        let next_haystack_char = if whole_word_search {
            // 所有内容（包括行尾）不是 [a-zA-Z0-9_] 的都被视为单词边界
            match self {
                SearchSource::Main(row) => row.columns.get(hidx + 1).map(|c| c.character),
                SearchSource::Tail(tail) => tail.columns.get(hidx + 1).map(|c| c.character),
            }
        } else {
            None // 不做全词搜索时不会被使用
        };
        (haystack_char, next_haystack_char)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchResult {
    // 我们在视口中已经找到的内容
    pub selections: Vec<Selection>,
    // 我们找到的选择中哪个当前是"活动的"（以不同方式高亮）
    pub active: Option<Selection>,
    // 我们在寻找什么
    pub needle: String,
    // 大小写是否重要？
    pub case_insensitive: bool,
    // 仅搜索整个单词，而不是单词内的部分
    pub whole_word_only: bool, // TODO
    // 如果我们没有更多行可搜索，从底部跳到顶部（或反之）
    pub wrap_search: bool,
}

impl SearchResult {
    /// 这仅用于 Debug 格式化 Grid，而 Grid 本身仅用于
    /// 测试。
    #[allow(clippy::ptr_arg)]
    pub(crate) fn mark_search_results_in_row(&self, row: &mut Cow<Row>, ridx: usize) {
        for s in &self.selections {
            if s.contains_row(ridx) {
                let replacement_char = if Some(s) == self.active.as_ref() {
                    '_'
                } else {
                    '#'
                };

                let (skip, take) = if ridx as isize == s.start.line() {
                    let skip = s.start.column();
                    let take = if s.end.line() == s.start.line() {
                        s.end.column() - s.start.column()
                    } else {
                        // 只标记行的其余部分。这个数字肯定太大了，但迭代器会处理这个
                        row.columns.len()
                    };
                    (skip, take)
                } else if ridx as isize == s.end.line() {
                    // 我们换行了，末尾在这一行，所以从开头取到末尾
                    (0, s.end.column())
                } else {
                    // 我们在中间（开头在上方，末尾在下方），所以全部标记
                    (0, row.columns.len())
                };

                row.to_mut()
                    .columns
                    .iter_mut()
                    .skip(skip)
                    .take(take)
                    .for_each(|x| *x = TerminalCharacter::new(replacement_char));
            }
        }
    }

    pub fn has_modifiers_set(&self) -> bool {
        self.wrap_search || self.whole_word_only || self.case_insensitive
    }

    fn check_if_haystack_char_matches_needle(
        &self,
        nidx: usize,
        needle_char: char,
        haystack_char: char,
        prev_haystack_char: Option<char>,
    ) -> bool {
        let mut chars_match = if self.case_insensitive {
            // 不区分大小写搜索
            // 目前只有 ascii，因为这个整个搜索函数无论如何都非常次优
            haystack_char.to_ascii_lowercase() == needle_char.to_ascii_lowercase()
        } else {
            // 区分大小写搜索
            haystack_char == needle_char
        };

        // 全词搜索
        // 只有当第一个不是匹配的干草堆字符是单词边界时，才是匹配
        if chars_match
            && self.whole_word_only
            && nidx == 0
            && !is_word_boundary(&prev_haystack_char)
        {
            // 匹配的开头不是单词边界，所以这不是命中
            chars_match = false;
        }

        chars_match
    }

    /// 搜索一行及其尾部。
    /// 尾部是 `row` 下方所有非规范行，`row` 本身不一定是规范的。
    pub(crate) fn search_row(&self, mut ridx: usize, row: &Row, tail: &[&Row]) -> Vec<Selection> {
        let mut res = Vec::new();
        if self.needle.is_empty() || row.columns.is_empty() {
            return res;
        }

        let mut tailit = tail.iter();
        let mut source = SearchSource::Main(row); // 我们当前从哪里获取干草堆字符
        let orig_ridx = ridx;
        let mut start = None; // 如果我们找到命中，这就是它开始的地方
        let mut nidx = 0; // 针索引
        let mut hidx = 0; // 干草堆索引
        let mut prev_haystack_char: Option<char> = None;
        loop {
            // 获取当前和下一个干草堆字符
            let (mut haystack_char, next_haystack_char) =
                source.get_next_two_chars(hidx, self.whole_word_only);

            // 获取当前针字符
            let needle_char = self.needle.chars().nth(nidx).unwrap(); // 这里解包是安全的

            // 检查针和干草堆是否匹配（带搜索选项）
            let chars_match = self.check_if_haystack_char_matches_needle(
                nidx,
                needle_char,
                haystack_char,
                prev_haystack_char,
            );

            if chars_match {
                // 如果针只有 1 长，下一个 `if` 也可能发生，所以我们不把它合并成一个大的 if-else
                if nidx == 0 {
                    start = Some(Position::new(ridx as i32, hidx as u16));
                }
                if nidx == self.needle.len() - 1 {
                    let mut end_found = true;
                    // 如果我们搜索全词，下一个非针字符需要是单词边界，
                    // 否则它不是命中（例如，较长单词内部的某个出现）。
                    if self.whole_word_only && !is_word_boundary(&next_haystack_char) {
                        // 匹配的末尾不是单词边界，所以这不是命中！
                        // 我们必须从开始的地方跳回（加一个字符）
                        nidx = 0;
                        ridx = start.unwrap().line() as usize;
                        hidx = start.unwrap().column(); // 将在下面递增
                        if start.unwrap().line() as usize == orig_ridx {
                            source = SearchSource::Main(row);
                            haystack_char = row.columns[hidx].character; // 以便 prev_char 被正确设置
                        } else {
                            // -1 来自主行
                            let tail_idx = start.unwrap().line() as usize - orig_ridx - 1;
                            // 我们也必须重置尾部迭代器。
                            tailit = tail[tail_idx..].iter();
                            let trow = tailit.next().unwrap();
                            haystack_char = trow.columns[hidx].character; // 以便 prev_char 被正确设置
                            source = SearchSource::Tail(trow);
                        }
                        start = None;
                        end_found = false;
                    }
                    if end_found {
                        let mut selection = Selection::default();
                        selection.start(start.unwrap());
                        selection.end(Position::new(ridx as i32, (hidx + 1) as u16));
                        res.push(selection);
                        nidx = 0;
                        if matches!(source, SearchSource::Tail(..)) {
                            // 搜索尾部时，我们只能找到一个额外的选择，所以在这里停止
                            break;
                        }
                    }
                } else {
                    nidx += 1;
                }
            } else {
                // 字符不匹配。从头开始搜索针
                start = None;
                nidx = 0;
                if matches!(source, SearchSource::Tail(..)) {
                    // 搜索尾部时发现不匹配，立即退出
                    break;
                }
            }

            hidx += 1;
            prev_haystack_char = Some(haystack_char);
            // 我们可能需要切换到尾部的新行
            if !source.get_next_source(&mut ridx, &mut hidx, &mut tailit, &start) {
                break;
            }
        }

        // 尾部可能还没有被换行（当来自 lines_below 时），
        // 所以末尾可能跨越比行宽更多的字符。
        // 因此我们需要重新排版末尾：
        for s in res.iter_mut() {
            while s.end.column() > row.width() {
                s.end.column.0 -= row.width();
                s.end.line.0 += 1;
            }
        }
        res
    }

    pub(crate) fn move_active_selection_to_next(&mut self) {
        if let Some(active_idx) = self.active {
            self.active = self
                .selections
                .iter()
                .skip_while(|s| *s != &active_idx)
                .nth(1)
                .cloned();
        } else {
            self.active = self.selections.first().cloned();
        }
    }

    pub(crate) fn move_active_selection_to_prev(&mut self) {
        if let Some(active_idx) = self.active {
            self.active = self
                .selections
                .iter()
                .rev()
                .skip_while(|s| *s != &active_idx)
                .nth(1)
                .cloned();
        } else {
            self.active = self.selections.last().cloned();
        }
    }

    pub(crate) fn unset_active_selection_if_nonexistent(&mut self) {
        if let Some(active_idx) = self.active {
            if !self.selections.contains(&active_idx) {
                self.active = None;
            }
        }
    }

    pub(crate) fn move_down(
        &mut self,
        amount: usize,
        viewport: &VecDeque<Row>,
        grid_height: usize,
    ) -> bool {
        let mut found_something = false;
        self.selections
            .iter_mut()
            .chain(self.active.iter_mut())
            .for_each(|x| x.move_down(amount));

        // 丢弃所有在新视口之外的搜索结果
        self.adjust_selections_to_moved_viewport(grid_height);

        // 在新行中搜索我们的针
        if !self.needle.is_empty() {
            if let Some(row) = viewport.front() {
                let mut tail = Vec::new();
                loop {
                    let tail_idx = 1 + tail.len();
                    if tail_idx < viewport.len() && !viewport[tail_idx].is_canonical {
                        tail.push(&viewport[tail_idx]);
                    } else {
                        break;
                    }
                }
                let selections = self.search_row(0, row, &tail);
                for selection in selections.iter().rev() {
                    self.selections.insert(0, *selection);
                    found_something = true;
                }
            }
        }
        found_something
    }

    pub(crate) fn move_up(
        &mut self,
        amount: usize,
        viewport: &VecDeque<Row>,
        lines_below: &VecDeque<Row>,
        grid_height: usize,
    ) -> bool {
        let mut found_something = false;
        self.selections
            .iter_mut()
            .chain(self.active.iter_mut())
            .for_each(|x| x.move_up(amount));
        // 丢弃所有在新视口之外的搜索结果
        self.adjust_selections_to_moved_viewport(grid_height);

        // 在新行中搜索我们的针
        if !self.needle.is_empty() {
            if let Some(row) = viewport.back() {
                let tail: Vec<&Row> = lines_below.iter().take_while(|r| !r.is_canonical).collect();
                let selections = self.search_row(viewport.len() - 1, row, &tail);
                for selection in selections {
                    // 我们只对从这个新行开始的结果感兴趣
                    if selection.start.line() as usize == viewport.len() - 1 {
                        self.selections.push(selection);
                        found_something = true;
                    }
                }
            }
        }
        found_something
    }

    fn adjust_selections_to_moved_viewport(&mut self, grid_height: usize) {
        // 丢弃所有在新视口之外的搜索结果
        self.selections
            .retain(|s| (s.start.line() as usize) < grid_height && s.end.line() >= 0);
        // 如果我们丢弃了活动元素，将其设置为 None
        self.unset_active_selection_if_nonexistent();
    }
}

impl Grid {
    pub fn search_down(&mut self) {
        self.search_scrollbuffer(SearchDirection::Down);
    }

    pub fn search_up(&mut self) {
        self.search_scrollbuffer(SearchDirection::Up);
    }

    pub fn clear_search(&mut self) {
        // 清除所有先前的高亮
        for res in &self.search_results.selections {
            self.output_buffer
                .update_lines(res.start.line() as usize, res.end.line() as usize);
        }
        self.search_results = Default::default();
    }

    pub fn set_search_string(&mut self, needle: &str) {
        self.search_results.needle = needle.to_string();
        self.search_viewport();
        // 如果当前视口不包含任何命中，
        // 我们来回跳转直到找到东西。从
        // 向后开始。
        if self.search_results.selections.is_empty() {
            self.search_up();
        }
        if self.search_results.selections.is_empty() {
            self.search_down();
        }
        // 我们在这个阶段仍然不想预选任何东西
        self.search_results.active = None;
        self.is_scrolled = true;
    }

    pub fn search_viewport(&mut self) {
        for ridx in 0..self.viewport.len() {
            let row = &self.viewport[ridx];
            let mut tail = Vec::new();
            loop {
                let tail_idx = ridx + tail.len() + 1;
                if tail_idx < self.viewport.len() && !self.viewport[tail_idx].is_canonical {
                    tail.push(&self.viewport[tail_idx]);
                } else {
                    break;
                }
            }
            let selections = self.search_results.search_row(ridx, row, &tail);
            for sel in &selections {
                // 强制转换有效，因为我们在这里不可能是负数
                self.output_buffer
                    .update_lines(sel.start.line() as usize, sel.end.line() as usize);
            }

            for selection in selections {
                self.search_results.selections.push(selection);
            }
        }
    }

    pub fn toggle_search_case_sensitivity(&mut self) {
        self.search_results.case_insensitive = !self.search_results.case_insensitive;
        for line in self.search_results.selections.drain(..) {
            self.output_buffer
                .update_lines(line.start.line() as usize, line.end.line() as usize);
        }
        self.search_viewport();
        // 也许我们之前的选择现在消失了
        self.search_results.unset_active_selection_if_nonexistent();
    }

    pub fn toggle_search_wrap(&mut self) {
        self.search_results.wrap_search = !self.search_results.wrap_search;
    }

    pub fn toggle_search_whole_words(&mut self) {
        self.search_results.whole_word_only = !self.search_results.whole_word_only;
        for line in self.search_results.selections.drain(..) {
            self.output_buffer
                .update_lines(line.start.line() as usize, line.end.line() as usize);
        }
        self.search_results.active = None;
        self.search_viewport();
        // 也许我们之前的选择现在消失了
        self.search_results.unset_active_selection_if_nonexistent();
    }

    fn search_scrollbuffer(&mut self, dir: SearchDirection) {
        let first_sel = self.search_results.selections.first();
        let last_sel = self.search_results.selections.last();

        let search_viewport_for_the_first_time =
            self.search_results.active.is_none() && !self.search_results.selections.is_empty();

        // 我们还没到末尾，所以可以迭代到当前视口内的下一个搜索结果
        let search_viewport_again = !self.search_results.selections.is_empty()
            && self.search_results.active.is_some()
            && match dir {
                SearchDirection::Up => self.search_results.active.as_ref() != first_sel,
                SearchDirection::Down => self.search_results.active.as_ref() != last_sel,
            };

        if search_viewport_for_the_first_time || search_viewport_again {
            // 我们可以留在视口中，只需移动活动选择
            self.search_viewport_again(search_viewport_for_the_first_time, dir);
        } else {
            // 需要移动视口
            let found_something = self.search_viewport_move(dir);

            // 我们没有找到任何东西，但我们被允许环绕
            if !found_something && self.search_results.wrap_search {
                self.search_viewport_wrap(dir);
            }
        }
    }

    fn search_viewport_again(
        &mut self,
        search_viewport_for_the_first_time: bool,
        dir: SearchDirection,
    ) {
        let new_active = match dir {
            SearchDirection::Up => self.search_results.selections.last().cloned().unwrap(),
            SearchDirection::Down => self.search_results.selections.first().cloned().unwrap(),
        };
        // 我们可以留在视口中，只需移动活动选择
        let active_idx = self.search_results.active.get_or_insert(new_active);
        self.output_buffer.update_lines(
            active_idx.start.line() as usize,
            active_idx.end.line() as usize,
        );
        if !search_viewport_for_the_first_time {
            match dir {
                SearchDirection::Up => self.search_results.move_active_selection_to_prev(),
                SearchDirection::Down => self.search_results.move_active_selection_to_next(),
            };
            if let Some(new_active) = self.search_results.active {
                self.output_buffer.update_lines(
                    new_active.start.line() as usize,
                    new_active.end.line() as usize,
                );
            }
        }
    }

    fn search_reached_opposite_end(&mut self, dir: SearchDirection) -> bool {
        match dir {
            SearchDirection::Up => self.lines_above.is_empty(),
            SearchDirection::Down => self.lines_below.is_empty(),
        }
    }

    fn search_viewport_move(&mut self, dir: SearchDirection) -> bool {
        // 我们需要移动视口
        let mut rows = 0;
        let mut found_something = false;

        // 如果找不到任何东西，我们可能会丢失当前选择
        let current_active_selection = self.search_results.active;
        while !found_something && !self.search_reached_opposite_end(dir) {
            rows += 1;
            found_something = match dir {
                SearchDirection::Up => self.scroll_up_one_line(),
                SearchDirection::Down => self.scroll_down_one_line(),
            };
        }

        if found_something {
            self.search_adjust_to_new_selection(dir);
        } else {
            // 我们没有找到东西，所以滚回开头
            for _ in 0..rows {
                match dir {
                    SearchDirection::Up => self.scroll_down_one_line(),
                    SearchDirection::Down => self.scroll_up_one_line(),
                };
            }
            self.search_results.active = current_active_selection;
        }
        found_something
    }

    fn search_adjust_to_new_selection(&mut self, dir: SearchDirection) {
        match dir {
            SearchDirection::Up => {
                self.search_results.move_active_selection_to_prev();
            },
            SearchDirection::Down => {
                // 我们可能需要再滚动一点，因为我们在搜索结果的开头，
                // 但末尾可能不可见
                if let Some(last) = self.search_results.selections.last() {
                    let distance = (last.end.line() - last.start.line()) as usize;
                    if distance < self.height {
                        for _ in 0..distance {
                            self.scroll_down_one_line();
                        }
                    }
                }
                self.search_results.move_active_selection_to_next();
            },
        }
        self.output_buffer.update_all_lines();
    }

    fn search_viewport_wrap(&mut self, dir: SearchDirection) {
        // 如果找不到任何东西，我们可能会丢失当前选择
        let current_active_selection = self.search_results.active;
        // 向上
        // 到对面的一端（向上搜索时到底部，向下搜索时到顶部）
        let mut rows = self.move_viewport_to_opposite_end(dir);

        // 我们在底部或顶部。也许我们已经在那里找到了东西
        // 如果没有，再滚回来，直到找到东西
        let mut found_something = match dir {
            SearchDirection::Up => self.search_results.selections.last().is_some(),
            SearchDirection::Down => self.search_results.selections.first().is_some(),
        };

        // 我们在回滚缓冲区的另一端没有找到任何东西，所以滚回来直到找到东西
        if !found_something {
            while rows >= 0 && !found_something {
                rows -= 1;
                found_something = match dir {
                    SearchDirection::Up => self.scroll_up_one_line(),
                    SearchDirection::Down => self.scroll_down_one_line(),
                };
            }
        }
        if found_something {
            self.search_results.active = match dir {
                SearchDirection::Up => self.search_results.selections.last().cloned(),
                SearchDirection::Down => {
                    // 我们需要滚动直到找到的项目在顶部
                    if let Some(first) = self.search_results.selections.first() {
                        for _ in 0..first.start.line() {
                            self.scroll_down_one_line();
                        }
                    }
                    self.search_results.selections.first().cloned()
                },
            };
            self.output_buffer.update_all_lines();
        } else {
            // 我们没有找到任何东西，所以重置旧的活动选择
            self.search_results.active = current_active_selection;
        }
    }

    fn move_viewport_to_opposite_end(&mut self, dir: SearchDirection) -> isize {
        let mut rows = 0;
        match dir {
            SearchDirection::Up => {
                // 到底部
                while !self.lines_below.is_empty() {
                    rows += 1;
                    self.scroll_down_one_line();
                }
            },
            SearchDirection::Down => {
                // 到顶部
                while !self.lines_above.is_empty() {
                    rows += 1;
                    self.scroll_up_one_line();
                }
            },
        }
        rows
    }
}
