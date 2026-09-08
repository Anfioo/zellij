pub const DEFAULT_CACHE_FILE_PATH: &str = "/tmp/status-bar-tips.cache";
pub const MAX_CACHE_HITS: usize = 20; //  这应该是 10，但目前有一个 bug，插件 load 函数被调用两次，因此缓存被命中两次
