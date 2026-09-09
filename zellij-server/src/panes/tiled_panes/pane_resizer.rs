use super::stacked_panes::StackedPanes;
use crate::{panes::PaneId, tab::Pane};
use kasuari::{Expression, Solver, Strength, Variable, WeightedRelation::EQ};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use zellij_utils::{
    errors::prelude::*,
    input::layout::SplitDirection,
    pane_size::{Constraint, Dimension, PaneGeom},
};

pub struct PaneResizer<'a> {
    panes: Rc<RefCell<HashMap<PaneId, &'a mut Box<dyn Pane>>>>,
    vars: HashMap<PaneId, Variable>,
    solver: Solver,
}

// FIXME: 直接持有一个可变的 Pane 引用而不是 PaneId、fixed、pos 和 size？
// 在窗格不再是 trait 对象之后再做这个！
#[derive(Debug, Clone, Copy)]
struct Span {
    pid: PaneId,
    direction: SplitDirection,
    pos: usize,
    size: Dimension,
    size_var: Variable,
}

type Grid = Vec<Vec<Span>>;

impl<'a> PaneResizer<'a> {
    pub fn new(panes: Rc<RefCell<HashMap<PaneId, &'a mut Box<dyn Pane>>>>) -> Self {
        let mut vars = HashMap::new();
        for &pane_id in panes.borrow().keys() {
            vars.insert(pane_id, Variable::new());
        }
        PaneResizer {
            panes,
            vars,
            solver: Solver::new(),
        }
    }

    pub fn layout(&mut self, direction: SplitDirection, space: usize) -> Result<()> {
        self.solver.reset();
        let grid = self
            .solve(direction, space)
            .map_err(|err| anyhow!("{}", err))?;
        let spans = self
            .discretize_spans(grid, space)
            .map_err(|err| anyhow!("{}", err))?;

        if self.is_layout_valid(&spans) {
            self.apply_spans(spans)?;
        }
        Ok(())
    }

    fn solve(&mut self, direction: SplitDirection, space: usize) -> Result<Grid, String> {
        let grid: Grid = self
            .grid_boundaries(direction)
            .into_iter()
            .map(|b| self.spans_in_boundary(direction, b))
            .collect();

        let constraints: HashSet<_> = grid
            .iter()
            .flat_map(|s| constrain_spans(space, s))
            .collect();

        self.solver
            .add_constraints(constraints)
            .map_err(|e| format!("{:?}", e))?;

        Ok(grid)
    }

    fn discretize_spans(&mut self, mut grid: Grid, space: usize) -> Result<Vec<Span>, String> {
        let mut rounded_sizes: HashMap<_, _> = grid
            .iter()
            .flatten()
            .map(|s| {
                (
                    s.size_var,
                    stable_round(self.solver.get_value(s.size_var)) as isize,
                )
            })
            .collect();

        // 将 f64 窗格大小四舍五入为 usize，不留间隙或重叠
        let mut finalised = Vec::new();
        for spans in &mut grid {
            let rounded_size: isize = spans.iter().map(|s| rounded_sizes[&s.size_var]).sum();
            let mut error = space as isize - rounded_size;
            let mut flex_spans: Vec<_> = spans
                .iter_mut()
                .filter(|s| !s.size.is_fixed() && !finalised.contains(&s.pid))
                .collect();
            flex_spans.sort_by_key(|s| rounded_sizes[&s.size_var]);
            if error < 0 {
                flex_spans.reverse();
            }
            for span in flex_spans {
                rounded_sizes
                    .entry(span.size_var)
                    .and_modify(|s| *s += error.signum());
                error -= error.signum();
            }
            finalised.extend(spans.iter().map(|s| s.pid));
        }

        // 根据四舍五入后的大小更新跨度位置
        for spans in &mut grid {
            let mut offset = 0;
            for span in spans {
                span.pos = offset;
                let sz = rounded_sizes[&span.size_var];
                if sz < 1 {
                    return Err("跨段空间已用完".into());
                }
                span.size.set_inner(sz as usize);
                offset += span.size.as_usize();
            }
        }

        Ok(grid.into_iter().flatten().collect())
    }

    // HACK: 这整个函数有点像 hack — 它在这里是为了防止我们在被给予
    // 一个糟糕的初始状态时破坏布局。如果此函数返回 false，则不会调整任何大小。
    fn is_layout_valid(&self, spans: &[Span]) -> bool {
        // 如果窗格堆叠太高无法容纳在屏幕上，在状态栏被卷入
        // 任何错误的调整大小之前放弃...
        for span in spans {
            let pane_is_stacked = self
                .panes
                .borrow()
                .get(&span.pid)
                .unwrap()
                .current_geom()
                .is_stacked();
            if pane_is_stacked && span.direction == SplitDirection::Vertical {
                let min_stack_height = StackedPanes::new(self.panes.clone())
                    .min_stack_height(&span.pid)
                    .unwrap();
                if span.size.as_usize() < min_stack_height {
                    return false;
                }
            }
        }
        true
    }

    fn apply_spans(&mut self, spans: Vec<Span>) -> Result<()> {
        let err_context = || format!("Failed to apply spans");
        let mut geoms_changed = false;
        for span in spans {
            let pane_is_stacked = self
                .panes
                .borrow()
                .get(&span.pid)
                .unwrap()
                .current_geom()
                .is_stacked();
            if pane_is_stacked {
                let current_geom = StackedPanes::new(self.panes.clone())
                    .position_and_size_of_stack(&span.pid)
                    .unwrap();
                let new_geom = match span.direction {
                    SplitDirection::Horizontal => PaneGeom {
                        x: span.pos,
                        cols: span.size,
                        ..current_geom
                    },
                    SplitDirection::Vertical => PaneGeom {
                        y: span.pos,
                        rows: span.size,
                        ..current_geom
                    },
                };
                StackedPanes::new(self.panes.clone()).resize_panes_in_stack(&span.pid, new_geom)?;
                if new_geom.rows.as_usize() != current_geom.rows.as_usize()
                    || new_geom.cols.as_usize() != current_geom.cols.as_usize()
                {
                    geoms_changed = true;
                }
            } else {
                let mut panes = self.panes.borrow_mut();
                let pane = panes.get_mut(&span.pid).unwrap();
                let current_geom = pane.position_and_size();
                let new_geom = match span.direction {
                    SplitDirection::Horizontal => PaneGeom {
                        x: span.pos,
                        cols: span.size,
                        ..pane.current_geom()
                    },
                    SplitDirection::Vertical => PaneGeom {
                        y: span.pos,
                        rows: span.size,
                        ..pane.current_geom()
                    },
                };
                if new_geom.rows.as_usize() != current_geom.rows.as_usize()
                    || new_geom.cols.as_usize() != current_geom.cols.as_usize()
                {
                    geoms_changed = true;
                }
                if pane.geom_override().is_some() {
                    pane.set_geom_override(new_geom);
                } else {
                    pane.set_geom(new_geom);
                }
            }
        }
        if geoms_changed {
            Ok(())
        } else {
            // 可能是四舍五入问题 - 这可能被认为是错误，取决于谁
            // 调用了我们 - 如果是显式调整大小操作，那显然是错误（用户
            // 想要调整大小且不关心百分比四舍五入），如果是调整整个
            // 终端窗口的大小，那可能不是
            Err(ZellijError::PaneSizeUnchanged).with_context(err_context)
        }
    }

    // FIXME: 这样的函数应该有单元测试！
    fn grid_boundaries(&self, direction: SplitDirection) -> Vec<(usize, usize)> {
        // 选择与调整大小方向 *垂直* 运行的跨度
        let spans: Vec<Span> = self
            .panes
            .borrow()
            .values()
            .filter_map(|p| self.get_span(!direction, p.as_ref()))
            .collect();

        let mut last_edge = 0;
        let mut bounds = Vec::new();
        let mut edges: Vec<usize> = spans.iter().map(|s| s.pos + s.size.as_usize()).collect();
        edges.sort_unstable();
        edges.dedup();
        for next in edges {
            let next_edge = next;
            bounds.push((last_edge, next_edge));
            last_edge = next_edge;
        }
        bounds
    }

    fn spans_in_boundary(&self, direction: SplitDirection, boundary: (usize, usize)) -> Vec<Span> {
        let bwn = |v, (s, e)| s <= v && v < e;
        let mut spans: Vec<_> = self
            .panes
            .borrow()
            .values()
            .filter(|p| match self.get_span(!direction, p.as_ref()) {
                Some(s) => {
                    let span_bounds = (s.pos, s.pos + s.size.as_usize());
                    bwn(span_bounds.0, boundary)
                        || (bwn(boundary.0, span_bounds)
                            && (bwn(boundary.1, span_bounds) || boundary.1 == span_bounds.1))
                },
                None => false,
            })
            .filter_map(|p| self.get_span(direction, p.as_ref()))
            .collect();
        spans.sort_unstable_by_key(|s| s.pos);
        spans
    }

    fn get_span(&self, direction: SplitDirection, pane: &dyn Pane) -> Option<Span> {
        let position_and_size = {
            let pas = pane.current_geom();
            if pas.is_stacked() && pas.rows.is_percent() {
                // 这是堆叠的主窗格
                StackedPanes::new(self.panes.clone()).position_and_size_of_stack(&pane.pid())
            } else if pas.is_stacked() {
                // 这是一个单行堆叠窗格，应该与堆叠的其余部分作为同一个 rect 处理，
                // 由上面 if 分支中的主窗格表示
                None
            } else {
                // 非堆叠窗格，正常处理
                Some(pas)
            }
        }?;
        let size_var = *self.vars.get(&pane.pid()).unwrap();
        match direction {
            SplitDirection::Horizontal => Some(Span {
                pid: pane.pid(),
                direction,
                pos: position_and_size.x,
                size: position_and_size.cols,
                size_var,
            }),
            SplitDirection::Vertical => Some(Span {
                pid: pane.pid(),
                direction,
                pos: position_and_size.y,
                size: position_and_size.rows,
                size_var,
            }),
        }
    }
}

fn constrain_spans(space: usize, spans: &[Span]) -> HashSet<kasuari::Constraint> {
    let mut constraints = HashSet::new();

    // 计算"灵活"空间（未被固定大小跨度消耗的空间）
    let new_flex_space = spans.iter().fold(space, |a, s| {
        if let Constraint::Fixed(sz) = s.size.constraint {
            a.saturating_sub(sz)
        } else {
            a
        }
    });

    // 跨度必须使用所有可用空间
    let full_size = spans
        .iter()
        .fold(Expression::from_constant(0.0), |acc, s| acc + s.size_var);
    constraints.insert(full_size.clone() | EQ(Strength::REQUIRED) | space as f64);

    // 尝试保持比例并锁定非灵活大小
    for span in spans {
        match span.size.constraint {
            Constraint::Fixed(s) => {
                constraints.insert(span.size_var | EQ(Strength::REQUIRED) | s as f64)
            },
            Constraint::Percent(p) => constraints.insert(
                (span.size_var / new_flex_space as f64) | EQ(Strength::STRONG) | (p / 100.0),
            ),
        };
    }

    constraints
}

fn stable_round(x: f64) -> f64 {
    ((x * 100.0).round() / 100.0).round()
}
