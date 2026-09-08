use crate::plugins::plugin_map::PluginMap;
use crate::plugins::wasm_bridge::PluginCache;
use crate::ClientId;
use crate::ThreadSenders;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use wasmi::Engine;

/// A 动态 线程 池 that pins 任务 to 特定 线程 based on plugin_id
/// 启动 with 1 线程 and expands when 线程 are busy, shrinks when 插件 卸载
pub struct PinnedExecutor {
    // Sparse vector - Some(线程) for 活动 线程, None for 移除的 线程
    execution_threads: Arc<Mutex<Vec<Option<ExecutionThread>>>>,

    // Maps plugin_id -> thread_index (永久 assignment)
    plugin_assignments: Arc<Mutex<HashMap<u32, usize>>>,

    // Maps thread_index -> set of plugin_ids 分配的 to it
    thread_plugins: Arc<Mutex<HashMap<usize, HashSet<u32>>>>,

    // Next 线程 index to use when spawning (monotonically increasing)
    next_thread_idx: AtomicUsize,

    // Maximum 线程 allowed
    max_threads: usize,

    // 状态 to send to 插件 (to be kept on execution 线程)
    senders: ThreadSenders,
    plugin_map: Arc<Mutex<PluginMap>>,
    connected_clients: Arc<Mutex<Vec<ClientId>>>,
    plugin_cache: PluginCache,
    engine: Engine,
}

struct ExecutionThread {
    sender: Sender<Job>,
    jobs_in_flight: Arc<AtomicUsize>, // Busy 状态 跟踪
}

enum Job {
    Work(
        Box<
            dyn FnOnce(
                    ThreadSenders,
                    Arc<Mutex<PluginMap>>,
                    Arc<Mutex<Vec<ClientId>>>,
                    PluginCache,
                    Engine,
                ) + Send
                + 'static,
        >,
    ),
    Shutdown, // Signal to 退出 the 工作线程 循环
}

impl PinnedExecutor {
    /// Creates a new pinned executor with the specified maximum number of 线程
    /// 启动 with exactly 1 线程
    pub fn new(
        max_threads: usize,
        senders: &ThreadSenders,
        plugin_map: &Arc<Mutex<PluginMap>>,
        connected_clients: &Arc<Mutex<Vec<ClientId>>>,
        plugin_cache: &PluginCache,
        engine: &Engine,
    ) -> Self {
        let max_threads = max_threads.max(1); // At least 1

        let thread_0 = Self::spawn_thread(
            0,
            senders.clone(),
            plugin_map.clone(),
            connected_clients.clone(),
            plugin_cache.clone(),
            engine.clone(),
        );

        PinnedExecutor {
            execution_threads: Arc::new(Mutex::new(vec![Some(thread_0)])),
            plugin_assignments: Arc::new(Mutex::new(HashMap::new())),
            thread_plugins: Arc::new(Mutex::new(HashMap::new())),
            next_thread_idx: AtomicUsize::new(1), // Next will be index 1
            max_threads,
            senders: senders.clone(),
            plugin_map: plugin_map.clone(),
            connected_clients: connected_clients.clone(),
            plugin_cache: plugin_cache.clone(),
            engine: engine.clone(),
        }
    }

    fn spawn_thread(
        thread_idx: usize,
        senders: ThreadSenders,
        plugin_map: Arc<Mutex<PluginMap>>,
        connected_clients: Arc<Mutex<Vec<ClientId>>>,
        plugin_cache: PluginCache,
        engine: Engine,
    ) -> ExecutionThread {
        let (sender, receiver) = channel::<Job>();
        let jobs_in_flight = Arc::new(AtomicUsize::new(0));
        let jobs_in_flight_clone = jobs_in_flight.clone();

        let thread_handle = thread::Builder::new()
            .name(format!("plugin-exec-{}", thread_idx))
            .spawn({
                move || {
                    let senders = senders;
                    let plugin_map = plugin_map;
                    let connected_clients = connected_clients;
                    let plugin_cache = plugin_cache;
                    let engine = engine;
                    while let Ok(job) = receiver.recv() {
                        match job {
                            Job::Work(work) => {
                                work(
                                    senders.clone(),
                                    plugin_map.clone(),
                                    connected_clients.clone(),
                                    plugin_cache.clone(),
                                    engine.clone(),
                                );
                                jobs_in_flight_clone.fetch_sub(1, Ordering::SeqCst);
                            },
                            Job::Shutdown => break,
                        }
                    }
                }
            });
        if let Err(e) = thread_handle {
            log::error!("Failed to spawn plugin execution thread: {}", e);
        }

        ExecutionThread {
            sender,
            jobs_in_flight,
        }
    }

    /// Register a 插件 and 分配 it to a 线程
    /// 调用的 from wasm_bridge when 加载 a 插件
    pub fn register_plugin(&self, plugin_id: u32) -> usize {
        let mut assignments = self.plugin_assignments.lock().unwrap();

        // If already 分配的 (shouldn't happen, but defensive)
        if let Some(&thread_idx) = assignments.get(&plugin_id) {
            return thread_idx;
        }

        let mut thread_plugins = self.thread_plugins.lock().unwrap();
        let threads = self.execution_threads.lock().unwrap();

        // 查找 a non-busy 线程 with 分配的 插件 (prefer 重用 线程)
        let mut best_thread: Option<(usize, usize)> = None; // (index, 加载)

        for (idx, thread_opt) in threads.iter().enumerate() {
            if let Some(thread) = thread_opt {
                let is_busy = thread.jobs_in_flight.load(Ordering::SeqCst) > 0;
                if !is_busy {
                    let load = thread_plugins.get(&idx).map(|s| s.len()).unwrap_or(0);
                    if best_thread.is_none() || best_thread.map(|b| load < b.1).unwrap_or(false) {
                        best_thread = Some((idx, load));
                    }
                }
            }
        }

        let thread_idx = if let Some((idx, _)) = best_thread {
            // 找到的 a non-busy 线程
            idx
        } else {
            // All 线程 are busy - need to expand
            if threads.len() < self.max_threads {
                // Spawn a new 线程
                let new_idx = self.next_thread_idx.fetch_add(1, Ordering::SeqCst);
                drop(threads); // 释放 锁 before spawning
                self.add_thread(new_idx);
                new_idx
            } else {
                // At max capacity, 分配 to least-加载的 线程
                threads
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, t)| t.as_ref().map(|_| idx))
                    .min_by_key(|&idx| thread_plugins.get(&idx).map(|s| s.len()).unwrap_or(0))
                    .unwrap_or_else(|| {
                        log::error!("Failed to find free thread to run the plugin!");
                        0 // this is a misconfiguration, but we don't want to crash the app
                          // if it happens
                    })
            }
        };

        // Update mappings
        assignments.insert(plugin_id, thread_idx);
        thread_plugins
            .entry(thread_idx)
            .or_insert_with(HashSet::new)
            .insert(plugin_id);

        thread_idx
    }

    fn add_thread(&self, thread_idx: usize) {
        let mut threads = self.execution_threads.lock().unwrap();
        let new_thread = Self::spawn_thread(
            thread_idx,
            self.senders.clone(),
            self.plugin_map.clone(),
            self.connected_clients.clone(),
            self.plugin_cache.clone(),
            self.engine.clone(),
        );

        // Extend vector if needed
        while threads.len() <= thread_idx {
            threads.push(None);
        }
        threads[thread_idx] = Some(new_thread);
    }

    /// Execute 任务 pinned to 插件's 分配的 线程
    pub fn execute_for_plugin<F>(&self, plugin_id: u32, f: F)
    where
        F: FnOnce(
                ThreadSenders,
                Arc<Mutex<PluginMap>>,
                Arc<Mutex<Vec<ClientId>>>,
                PluginCache,
                Engine,
            ) + Send
            + 'static,
    {
        // Look up 分配的 线程
        let thread_idx = {
            let assignments = self.plugin_assignments.lock().unwrap();
            assignments.get(&plugin_id).copied()
        };
        let Some(thread_idx) = thread_idx else {
            log::error!("Failed to find thread for plugin with id: {}", plugin_id);
            return;
        };

        // Get 线程 and mark as busy
        let threads = self.execution_threads.lock().unwrap();
        let thread = threads[thread_idx].as_ref();
        let Some(thread) = thread else {
            log::error!("Failed to find thread for plugin with id: {}", plugin_id);
            return;
        };

        // Increment busy 计数器 BEFORE sending 工作
        thread.jobs_in_flight.fetch_add(1, Ordering::SeqCst);

        // Send 工作
        let job = Job::Work(Box::new(f));
        if let Err(_) = thread.sender.send(job) {
            // 线程 died unexpectedly - this is a critical 错误
            thread.jobs_in_flight.fetch_sub(1, Ordering::SeqCst);
            log::error!("Plugin executor thread {} has died", thread_idx);
        }
    }

    /// 加载 a 插件: register it and execute the 加载 工作 on its 分配的 线程
    /// This combines registration + execution for 插件 加载
    pub fn execute_plugin_load<F>(&self, plugin_id: u32, f: F)
    where
        F: FnOnce(
                ThreadSenders,
                Arc<Mutex<PluginMap>>,
                Arc<Mutex<Vec<ClientId>>>,
                PluginCache,
                Engine,
            ) + Send
            + 'static,
    {
        // Register 插件 and 分配 to a 线程
        self.register_plugin(plugin_id);

        // Execute the 加载 工作 on the 分配的 线程
        self.execute_for_plugin(plugin_id, f);
    }

    /// 卸载 a 插件: execute 清理 工作, then unregister and potentially shrink 池
    /// This combines 清理 execution + unregistration for 插件 卸载
    /// Requires ARC<Self> so we can clone it into the 闭包 for unregistration
    pub fn execute_plugin_unload(
        self: &Arc<Self>,
        plugin_id: u32,
        f: impl FnOnce(
                ThreadSenders,
                Arc<Mutex<PluginMap>>,
                Arc<Mutex<Vec<ClientId>>>,
                PluginCache,
                Engine,
            ) + Send
            + 'static,
    ) {
        let executor = self.clone();
        self.execute_for_plugin(
            plugin_id,
            move |senders, plugin_map, connected_clients, plugin_cache, engine| {
                // Execute the 清理 工作
                f(senders, plugin_map, connected_clients, plugin_cache, engine);

                // Unregister 插件 and potentially shrink the 池
                executor.unregister_plugin(plugin_id);
            },
        );
    }

    /// Unregister a 插件 and potentially shrink the 池
    /// 调用的 from wasm_bridge after 插件 清理 is complete
    pub fn unregister_plugin(&self, plugin_id: u32) {
        let mut assignments = self.plugin_assignments.lock().unwrap();
        let mut thread_plugins = self.thread_plugins.lock().unwrap();

        if let Some(thread_idx) = assignments.remove(&plugin_id) {
            if let Some(plugins) = thread_plugins.get_mut(&thread_idx) {
                plugins.remove(&plugin_id);
            }
        }

        drop(assignments);
        drop(thread_plugins);

        // 尝试 to shrink the 池
        self.try_shrink_pool();
    }

    fn try_shrink_pool(&self) {
        let mut threads = self.execution_threads.lock().unwrap();
        let thread_plugins = self.thread_plugins.lock().unwrap();

        // 查找 线程 with no 分配的 插件 (except 线程 0, always keep it)
        let threads_to_remove: Vec<usize> = threads
            .iter()
            .enumerate()
            .skip(1) // Never 移除 线程 0
            .filter_map(|(idx, thread_opt)| {
                if thread_opt.is_some() {
                    let has_plugins = thread_plugins
                        .get(&idx)
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);
                    if !has_plugins {
                        Some(idx)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Shutdown and 移除 idle 线程
        for idx in threads_to_remove {
            if let Some(thread) = threads[idx].take() {
                let _ = thread.sender.send(Job::Shutdown);
            }
        }
    }

    #[cfg(test)]
    pub fn thread_count(&self) -> usize {
        self.execution_threads
            .lock()
            .unwrap()
            .iter()
            .filter(|t| t.is_some())
            .count()
    }
}

impl Drop for PinnedExecutor {
    fn drop(&mut self) {
        let mut threads = self.execution_threads.lock().unwrap();

        // Send shutdown to all 线程
        for thread_opt in threads.iter_mut() {
            if let Some(thread) = thread_opt {
                let _ = thread.sender.send(Job::Shutdown);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{channel, Sender};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;
    use std::time::Duration;

    // 测试 fixtures
    fn create_test_dependencies() -> (
        ThreadSenders,
        Arc<Mutex<PluginMap>>,
        Arc<Mutex<Vec<ClientId>>>,
        PluginCache,
        Engine,
    ) {
        use std::path::PathBuf;
        use wasmi::Module;
        use zellij_utils::channels::{self, SenderWithContext};

        let (send_to_pty, _receive_pty) = channels::bounded(1);
        let (send_to_screen, _receive_screen) = channels::bounded(1);
        let (send_to_plugin, _receive_plugin) = channels::bounded(1);
        let (send_to_server, _receive_server) = channels::bounded(1);
        let (send_to_pty_writer, _receive_pty_writer) = channels::bounded(1);
        let (send_to_background_jobs, _receive_background_jobs) = channels::bounded(1);

        let to_pty = SenderWithContext::new(send_to_pty);
        let to_screen = SenderWithContext::new(send_to_screen);
        let to_plugin = SenderWithContext::new(send_to_plugin);
        let to_server = SenderWithContext::new(send_to_server);
        let to_pty_writer = SenderWithContext::new(send_to_pty_writer);
        let to_background_jobs = SenderWithContext::new(send_to_background_jobs);

        let senders = ThreadSenders {
            to_pty: Some(to_pty),
            to_screen: Some(to_screen),
            to_plugin: Some(to_plugin),
            to_server: Some(to_server),
            to_pty_writer: Some(to_pty_writer),
            to_background_jobs: Some(to_background_jobs),
            should_silently_fail: false,
        };

        let plugin_map = Arc::new(Mutex::new(PluginMap::default()));
        let connected_clients = Arc::new(Mutex::new(vec![]));

        let plugin_cache = Arc::new(Mutex::new(
            std::collections::HashMap::<PathBuf, Module>::new(),
        ));

        let engine = Engine::default();

        (senders, plugin_map, connected_clients, plugin_cache, engine)
    }

    fn create_test_executor(max_threads: usize) -> Arc<PinnedExecutor> {
        let (senders, plugin_map, clients, cache, engine) = create_test_dependencies();
        Arc::new(PinnedExecutor::new(
            max_threads,
            &senders,
            &plugin_map,
            &clients,
            &cache,
            &engine,
        ))
    }

    // Helper to 创建 a 任务 that signals completion via 通道
    fn make_signaling_job(
        tx: Sender<()>,
    ) -> impl FnOnce(
        ThreadSenders,
        Arc<Mutex<PluginMap>>,
        Arc<Mutex<Vec<ClientId>>>,
        PluginCache,
        Engine,
    ) + Send
           + 'static {
        move |_senders, _plugin_map, _clients, _cache, _engine| {
            tx.send(()).unwrap();
        }
    }

    // Helper to 验证 线程 assignment by capturing 线程 name in 任务
    fn get_thread_name_for_plugin(executor: &Arc<PinnedExecutor>, plugin_id: u32) -> String {
        let (tx, rx) = channel();
        executor.execute_for_plugin(plugin_id, move |_s, _p, _c, _ca, _e| {
            let name = thread::current().name().unwrap().to_string();
            tx.send(name).unwrap();
        });
        rx.recv_timeout(Duration::from_secs(5))
            .expect("Thread name should be received")
    }

    #[test]
    fn test_new_creates_one_thread() {
        let executor = create_test_executor(4);
        assert_eq!(executor.thread_count(), 1);
    }

    #[test]
    fn test_new_respects_min_threads() {
        let executor = create_test_executor(0);
        assert_eq!(
            executor.thread_count(),
            1,
            "Executor should enforce minimum of 1 thread"
        );
    }

    #[test]
    fn test_first_plugin_assigned_to_thread_zero() {
        let executor = create_test_executor(4);
        let thread_idx = executor.register_plugin(1);
        assert_eq!(thread_idx, 0);
    }

    #[test]
    fn test_multiple_plugins_share_thread_when_idle() {
        let executor = create_test_executor(4);
        let thread_idx1 = executor.register_plugin(1);
        let thread_idx2 = executor.register_plugin(2);
        assert_eq!(thread_idx1, 0);
        assert_eq!(
            thread_idx2, 0,
            "Second plugin should share thread 0 when idle"
        );
    }

    #[test]
    fn test_new_thread_spawns_when_all_busy() {
        let executor = create_test_executor(3);

        // Register 插件 1, gets 线程 0
        let thread_idx1 = executor.register_plugin(1);
        assert_eq!(thread_idx1, 0);

        // Make 线程 0 busy with a barrier
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier_clone.wait();
        });

        // Give the 任务 a moment to 启动 executing and block
        thread::sleep(Duration::from_millis(50));

        // Register 插件 2 while 线程 0 is busy
        let thread_idx2 = executor.register_plugin(2);
        assert_eq!(
            thread_idx2, 1,
            "Plugin 2 should get new thread 1 when thread 0 is busy"
        );

        // 验证 线程 count
        assert_eq!(executor.thread_count(), 2);

        // 释放 barrier
        barrier.wait();
    }

    #[test]
    fn test_respects_max_threads_limit() {
        let executor = create_test_executor(2);

        // Register 插件 1, gets 线程 0
        executor.register_plugin(1);

        // Make 线程 0 busy
        let barrier1 = Arc::new(Barrier::new(2));
        let barrier1_clone = barrier1.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier1_clone.wait();
        });
        thread::sleep(Duration::from_millis(50));

        // Register 插件 2, gets 线程 1
        executor.register_plugin(2);

        // Make 线程 1 busy
        let barrier2 = Arc::new(Barrier::new(2));
        let barrier2_clone = barrier2.clone();
        executor.execute_for_plugin(2, move |_s, _p, _c, _ca, _e| {
            barrier2_clone.wait();
        });
        thread::sleep(Duration::from_millis(50));

        // Register 插件 3 when all 线程 busy
        let thread_idx3 = executor.register_plugin(3);
        assert!(
            thread_idx3 == 0 || thread_idx3 == 1,
            "Plugin 3 should be assigned to existing thread"
        );
        assert_eq!(executor.thread_count(), 2, "Should not exceed max_threads");

        // 释放 barriers
        barrier1.wait();
        barrier2.wait();
    }

    #[test]
    fn test_duplicate_registration_returns_same_thread() {
        let executor = create_test_executor(4);
        let thread_idx1 = executor.register_plugin(1);
        let thread_idx2 = executor.register_plugin(1);
        assert_eq!(
            thread_idx1, thread_idx2,
            "Duplicate registration should return same thread"
        );
    }

    #[test]
    fn test_load_balancing_prefers_least_loaded() {
        let executor = create_test_executor(3);

        // Register 插件 1, 2, 3 to 线程 0 (when idle)
        executor.register_plugin(1);
        executor.register_plugin(2);
        executor.register_plugin(3);

        // Make 线程 0 busy
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier_clone.wait();
        });
        thread::sleep(Duration::from_millis(50));

        // Register 插件 4 while 线程 0 is busy (spawns 线程 1)
        let thread_idx4 = executor.register_plugin(4);
        assert_eq!(thread_idx4, 1);

        // 释放 barrier
        barrier.wait();
        thread::sleep(Duration::from_millis(50));

        // Register 插件 5 when both 线程 idle
        // 线程 0 has 3 插件, 线程 1 has 1 插件
        let thread_idx5 = executor.register_plugin(5);
        assert_eq!(
            thread_idx5, 1,
            "Plugin 5 should be assigned to less loaded thread 1"
        );
    }

    #[test]
    fn test_execute_for_plugin_runs_on_correct_thread() {
        let executor = create_test_executor(3);

        // Register 插件 1 to 线程 0
        executor.register_plugin(1);

        // Make 线程 0 busy to force 插件 2 to 线程 1
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier_clone.wait();
        });
        thread::sleep(Duration::from_millis(50));

        // Register 插件 2 to 线程 1
        executor.register_plugin(2);

        // 释放 barrier
        barrier.wait();
        thread::sleep(Duration::from_millis(50));

        // Get 线程 names for both 插件
        let thread_name1 = get_thread_name_for_plugin(&executor, 1);
        let thread_name2 = get_thread_name_for_plugin(&executor, 2);

        assert_eq!(thread_name1, "plugin-exec-0");
        assert_eq!(thread_name2, "plugin-exec-1");
    }

    #[test]
    fn test_execute_for_plugin_unregistered() {
        let executor = create_test_executor(4);
        let (tx, rx) = channel();

        // Execute 任务 for unregistered 插件
        executor.execute_for_plugin(999, make_signaling_job(tx));

        // 尝试 to receive with 超时 - should 超时
        let result = rx.recv_timeout(Duration::from_millis(100));
        assert!(
            result.is_err(),
            "Job for unregistered plugin should not execute"
        );
    }

    #[test]
    fn test_job_execution_order_per_thread() {
        let executor = create_test_executor(4);
        executor.register_plugin(1);

        let order = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = channel();

        // Execute 3 任务 for 插件 1
        for i in 1..=3 {
            let order_clone = order.clone();
            let tx_clone = tx.clone();
            executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
                order_clone.lock().unwrap().push(i);
                tx_clone.send(()).unwrap();
            });
        }

        // Wait for all 3 任务 to complete
        for _ in 0..3 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Job should complete");
        }

        assert_eq!(
            *order.lock().unwrap(),
            vec![1, 2, 3],
            "Jobs should execute in order"
        );
    }

    #[test]
    fn test_jobs_complete_successfully() {
        let executor = create_test_executor(4);
        executor.register_plugin(1);

        let (tx, rx) = channel();
        executor.execute_for_plugin(1, make_signaling_job(tx));

        let result = rx.recv_timeout(Duration::from_secs(5));
        assert!(result.is_ok(), "Job should complete successfully");
    }

    #[test]
    fn test_concurrent_jobs_on_different_threads() {
        let executor = create_test_executor(3);

        // Register 插件 1 to 线程 0
        executor.register_plugin(1);

        // Make 线程 0 busy to force 插件 2 to 线程 1
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier_clone.wait();
        });
        thread::sleep(Duration::from_millis(50));

        // Register 插件 2 to 线程 1
        executor.register_plugin(2);

        // 释放 barrier
        barrier.wait();
        thread::sleep(Duration::from_millis(50));

        // Execute 任务 on both 线程 concurrently
        let sync_barrier = Arc::new(Barrier::new(3)); // 2 任务 + 测试 线程
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        let barrier1 = sync_barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier1.wait();
            tx1.send(()).unwrap();
        });

        let barrier2 = sync_barrier.clone();
        executor.execute_for_plugin(2, move |_s, _p, _c, _ca, _e| {
            barrier2.wait();
            tx2.send(()).unwrap();
        });

        // 释放 both 任务 simultaneously
        sync_barrier.wait();

        // Both should complete
        assert!(rx1.recv_timeout(Duration::from_secs(5)).is_ok());
        assert!(rx2.recv_timeout(Duration::from_secs(5)).is_ok());
    }

    #[test]
    fn test_execute_plugin_load_registers_and_executes() {
        let executor = create_test_executor(4);

        let (tx, rx) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx));

        // Wait for 加载 to complete
        rx.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        // 验证 插件 is registered
        let thread_idx = executor.register_plugin(1);
        assert_eq!(thread_idx, 0, "Plugin should already be registered");
    }

    #[test]
    fn test_execute_plugin_unload_runs_cleanup_before_unregister() {
        let executor = create_test_executor(4);

        // 加载 插件
        let (tx_load, rx_load) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx_load));
        rx_load
            .recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        // 卸载 插件 with 清理
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let (tx_unload, rx_unload) = channel();

        executor.execute_plugin_unload(1, move |_s, _p, _c, _ca, _e| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            tx_unload.send(()).unwrap();
        });

        // Wait for 卸载 to complete
        rx_unload
            .recv_timeout(Duration::from_secs(5))
            .expect("Unload should complete");

        // 验证 清理 ran
        assert_eq!(counter.load(Ordering::SeqCst), 1, "Cleanup should have run");

        // Give unregister a moment to complete
        thread::sleep(Duration::from_millis(100));

        // 插件 should be unregistered - registering again should 分配 new 线程
        let thread_idx = executor.register_plugin(1);
        assert_eq!(thread_idx, 0, "Plugin should be re-registered to thread 0");
    }

    #[test]
    fn test_unload_sequence_is_correct() {
        let executor = create_test_executor(4);

        // 加载 插件
        let (tx_load, rx_load) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx_load));
        rx_load
            .recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        // 卸载 with sequence 跟踪
        let sequence = Arc::new(Mutex::new(Vec::new()));
        let sequence_clone = sequence.clone();
        let (tx_unload, rx_unload) = channel();

        executor.execute_plugin_unload(1, move |_s, _p, _c, _ca, _e| {
            sequence_clone.lock().unwrap().push("cleanup");
            tx_unload.send(()).unwrap();
        });

        // Wait for 卸载 to complete
        rx_unload
            .recv_timeout(Duration::from_secs(5))
            .expect("Unload should complete");
        sequence.lock().unwrap().push("after");

        assert_eq!(*sequence.lock().unwrap(), vec!["cleanup", "after"]);
    }

    #[test]
    fn test_shrink_removes_idle_threads() {
        let executor = create_test_executor(4);

        // 加载 插件 1 to 线程 0
        let (tx1, rx1) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx1));
        rx1.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        // Make 线程 0 busy to force 插件 2 to 线程 1
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier_clone.wait();
        });
        thread::sleep(Duration::from_millis(50));

        // 加载 插件 2 to 线程 1
        let (tx2, rx2) = channel();
        executor.execute_plugin_load(2, make_signaling_job(tx2));
        rx2.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        // 释放 barrier
        barrier.wait();
        thread::sleep(Duration::from_millis(50));

        let thread_count_before = executor.thread_count();
        assert!(thread_count_before >= 2, "Should have at least 2 threads");

        // 卸载 插件 2
        let (tx_unload2, rx_unload2) = channel();
        executor.execute_plugin_unload(2, make_signaling_job(tx_unload2));
        rx_unload2
            .recv_timeout(Duration::from_secs(5))
            .expect("Unload should complete");

        // Give shrinking a moment to complete
        thread::sleep(Duration::from_millis(100));

        // 线程 count should decrease after 卸载
        let thread_count_after = executor.thread_count();
        assert!(
            thread_count_after < thread_count_before,
            "Idle threads should be removed"
        );
        assert!(thread_count_after >= 1, "Thread 0 should remain");
    }

    #[test]
    fn test_thread_zero_never_removed() {
        let executor = create_test_executor(4);

        // 加载 a 插件 and then 卸载 it
        let (tx_load, rx_load) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx_load));
        rx_load
            .recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        let (tx_unload, rx_unload) = channel();
        executor.execute_plugin_unload(1, make_signaling_job(tx_unload));
        rx_unload
            .recv_timeout(Duration::from_secs(5))
            .expect("Unload should complete");

        // Give shrinking a moment
        thread::sleep(Duration::from_millis(100));

        // 线程 0 should remain
        assert!(
            executor.thread_count() >= 1,
            "Thread 0 should never be removed"
        );
    }

    #[test]
    fn test_active_threads_not_removed() {
        let executor = create_test_executor(4);

        // 加载 插件 1 to 线程 0
        let (tx1, rx1) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx1));
        rx1.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        // Force 插件 2 to 线程 1 by making 线程 0 busy
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier_clone.wait();
        });
        thread::sleep(Duration::from_millis(50));

        let (tx2, rx2) = channel();
        executor.execute_plugin_load(2, make_signaling_job(tx2));
        rx2.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        barrier.wait();
        thread::sleep(Duration::from_millis(100));

        let thread_count_with_both = executor.thread_count();
        assert!(
            thread_count_with_both >= 2,
            "Should have at least 2 threads with 2 plugins"
        );

        // 卸载 插件 2
        let (tx_unload, rx_unload) = channel();
        executor.execute_plugin_unload(2, make_signaling_job(tx_unload));
        rx_unload
            .recv_timeout(Duration::from_secs(5))
            .expect("Unload should complete");

        thread::sleep(Duration::from_millis(100));

        // 插件 1's 线程 should still 工作 (验证 活动 线程 not affected)
        let (tx_test, rx_test) = channel();
        executor.execute_for_plugin(1, make_signaling_job(tx_test));
        assert!(
            rx_test.recv_timeout(Duration::from_secs(5)).is_ok(),
            "Plugin 1's thread should still work"
        );

        // 线程 count should decrease after 卸载
        let thread_count_after = executor.thread_count();
        assert!(
            thread_count_after < thread_count_with_both,
            "Idle thread should be removed"
        );
        assert!(thread_count_after >= 1, "Active threads should remain");
    }

    #[test]
    fn test_shrink_does_not_affect_remaining_threads() {
        let executor = create_test_executor(4);

        // 加载 插件 1 to 线程 0
        let (tx1, rx1) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx1));
        rx1.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        // Force 插件 2 to 线程 1
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier_clone.wait();
        });
        thread::sleep(Duration::from_millis(50));

        let (tx2, rx2) = channel();
        executor.execute_plugin_load(2, make_signaling_job(tx2));
        rx2.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        barrier.wait();
        thread::sleep(Duration::from_millis(50));

        // 卸载 插件 2 (shrinks 池)
        let (tx_unload, rx_unload) = channel();
        executor.execute_plugin_unload(2, make_signaling_job(tx_unload));
        rx_unload
            .recv_timeout(Duration::from_secs(5))
            .expect("Unload should complete");

        thread::sleep(Duration::from_millis(100));

        // Execute 任务 for 插件 1
        let (tx_test, rx_test) = channel();
        executor.execute_for_plugin(1, make_signaling_job(tx_test));

        assert!(
            rx_test.recv_timeout(Duration::from_secs(5)).is_ok(),
            "Plugin 1's thread should still work"
        );
    }

    #[test]
    fn test_drop_cleans_up_gracefully() {
        let executor = create_test_executor(4);

        // 加载 multiple 插件 on different 线程
        let (tx1, rx1) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx1));
        rx1.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier_clone.wait();
        });
        thread::sleep(Duration::from_millis(50));

        let (tx2, rx2) = channel();
        executor.execute_plugin_load(2, make_signaling_job(tx2));
        rx2.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        barrier.wait();

        // 丢弃 executor
        drop(executor);

        // 测试 completes without panic
    }

    #[test]
    fn test_drop_with_jobs_in_flight() {
        let executor = create_test_executor(4);
        executor.register_plugin(1);

        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        executor.execute_for_plugin(1, move |_s, _p, _c, _ca, _e| {
            barrier_clone.wait();
        });

        // 丢弃 executor while 任务 is blocked
        drop(executor);

        // 释放 barrier (任务 may or may not complete, but shouldn't panic)
        barrier.wait();

        // 测试 completes without panic
    }

    #[test]
    fn test_concurrent_plugin_registrations() {
        let executor = create_test_executor(4);

        let handles: Vec<_> = (1..=10)
            .map(|i| {
                let exec = executor.clone();
                thread::spawn(move || exec.register_plugin(i))
            })
            .collect();

        for handle in handles {
            let thread_idx = handle.join().expect("Thread should not panic");
            assert!(thread_idx < 4, "Thread index should be valid");
        }
    }

    #[test]
    fn test_unregister_nonexistent_plugin() {
        let executor = create_test_executor(4);

        // Unregister non-existent 插件
        executor.unregister_plugin(999);

        // Executor should still 工作
        let (tx, rx) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx));
        assert!(rx.recv_timeout(Duration::from_secs(5)).is_ok());
    }

    #[test]
    fn test_max_threads_one() {
        let executor = create_test_executor(1);

        // Register multiple 插件
        let thread_idx1 = executor.register_plugin(1);
        let thread_idx2 = executor.register_plugin(2);
        let thread_idx3 = executor.register_plugin(3);

        assert_eq!(thread_idx1, 0);
        assert_eq!(thread_idx2, 0);
        assert_eq!(thread_idx3, 0);
        assert_eq!(executor.thread_count(), 1);
    }

    #[test]
    fn test_rapid_load_unload_cycles() {
        let executor = create_test_executor(4);

        // 加载 插件 1
        let (tx1, rx1) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx1));
        rx1.recv_timeout(Duration::from_secs(5))
            .expect("Load should complete");

        // 卸载 插件 1
        let (tx2, rx2) = channel();
        executor.execute_plugin_unload(1, make_signaling_job(tx2));
        rx2.recv_timeout(Duration::from_secs(5))
            .expect("Unload should complete");

        thread::sleep(Duration::from_millis(100));

        // 加载 插件 1 again
        let (tx3, rx3) = channel();
        executor.execute_plugin_load(1, make_signaling_job(tx3));
        rx3.recv_timeout(Duration::from_secs(5))
            .expect("Second load should complete");

        // Execute 任务 for 插件 1
        let (tx4, rx4) = channel();
        executor.execute_for_plugin(1, make_signaling_job(tx4));
        assert!(
            rx4.recv_timeout(Duration::from_secs(5)).is_ok(),
            "Executor should handle cycles correctly"
        );
    }

    #[test]
    fn test_many_plugins_limited_threads() {
        let executor = create_test_executor(4);
        let (tx, rx) = channel();

        // 加载 20 插件
        for i in 1..=20 {
            let tx_clone = tx.clone();
            executor.execute_plugin_load(i, make_signaling_job(tx_clone));
        }

        // 收集 20 completion signals
        for _ in 1..=20 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Load should complete");
        }

        assert!(
            executor.thread_count() <= 4,
            "Should not exceed max_threads"
        );
    }

    #[test]
    fn test_full_lifecycle() {
        let executor = create_test_executor(4);
        let (tx, rx) = channel();

        // 加载 5 插件
        for i in 1..=5 {
            let tx_clone = tx.clone();
            executor.execute_plugin_load(i, make_signaling_job(tx_clone));
        }

        // Wait for all 加载
        for _ in 0..5 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Load should complete");
        }

        // Execute 2 任务 per 插件 (10 total)
        for i in 1..=5 {
            for _ in 0..2 {
                let tx_clone = tx.clone();
                executor.execute_for_plugin(i, make_signaling_job(tx_clone));
            }
        }

        // Wait for all 任务
        for _ in 0..10 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Job should complete");
        }

        let thread_count_before = executor.thread_count();

        // 卸载 3 插件
        for i in 1..=3 {
            let tx_clone = tx.clone();
            executor.execute_plugin_unload(i, make_signaling_job(tx_clone));
        }

        // Wait for unloads
        for _ in 0..3 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Unload should complete");
        }

        thread::sleep(Duration::from_millis(100));

        // 线程 count should decrease or stay the same
        let thread_count_after = executor.thread_count();
        assert!(
            thread_count_after <= thread_count_before,
            "Thread count should decrease after unloads"
        );

        // Execute 任务 for remaining 插件
        for i in 4..=5 {
            let tx_clone = tx.clone();
            executor.execute_for_plugin(i, make_signaling_job(tx_clone));
        }

        for _ in 0..2 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Job should complete");
        }

        // 丢弃 executor
        drop(executor);
    }

    #[test]
    fn test_realistic_plugin_churn() {
        let executor = create_test_executor(4);
        let (tx, rx) = channel();

        // 加载 插件 1, 2, 3
        for i in 1..=3 {
            let tx_clone = tx.clone();
            executor.execute_plugin_load(i, make_signaling_job(tx_clone));
        }
        for _ in 0..3 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Load should complete");
        }

        // Execute 任务 for each
        for i in 1..=3 {
            let tx_clone = tx.clone();
            executor.execute_for_plugin(i, make_signaling_job(tx_clone));
        }
        for _ in 0..3 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Job should complete");
        }

        // 卸载 插件 2
        let tx_clone = tx.clone();
        executor.execute_plugin_unload(2, make_signaling_job(tx_clone));
        rx.recv_timeout(Duration::from_secs(5))
            .expect("Unload should complete");

        thread::sleep(Duration::from_millis(100));

        // 加载 插件 4, 5
        for i in 4..=5 {
            let tx_clone = tx.clone();
            executor.execute_plugin_load(i, make_signaling_job(tx_clone));
        }
        for _ in 0..2 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Load should complete");
        }

        // Execute 任务 for 插件 1, 3, 4, 5
        for i in &[1, 3, 4, 5] {
            let tx_clone = tx.clone();
            executor.execute_for_plugin(*i, make_signaling_job(tx_clone));
        }
        for _ in 0..4 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Job should complete");
        }

        // 卸载 插件 1, 3
        for i in &[1, 3] {
            let tx_clone = tx.clone();
            executor.execute_plugin_unload(*i, make_signaling_job(tx_clone));
        }
        for _ in 0..2 {
            rx.recv_timeout(Duration::from_secs(5))
                .expect("Unload should complete");
        }

        thread::sleep(Duration::from_millis(100));

        // 验证 线程 count reflects 活动 插件 (4 and 5)
        assert!(
            executor.thread_count() >= 1,
            "Should have at least thread 0"
        );

        drop(executor);
    }
}
