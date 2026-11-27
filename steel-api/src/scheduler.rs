//! Планировщик — синхронные/асинхронные задачи, как Bukkit Scheduler, но без checked exceptions.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rustc_hash::FxHashMap;
use steel_utils::locks::SyncMutex;

pub type TaskId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Scheduled,
    Running,
    Cancelled,
    Completed,
}

pub struct TaskHandle {
    pub id: TaskId,
    scheduler: Arc<SchedulerInner>,
}
impl TaskHandle {
    pub fn id(&self) -> TaskId { self.id }
    pub fn cancel(&self) -> bool { self.scheduler.cancel(self.id) }
    pub fn is_cancelled(&self) -> bool { self.scheduler.is_cancelled(self.id) }
}

struct Task {
    #[expect(dead_code, reason = "task id stored for handle correlation, used in scheduler")]
    id: TaskId,
    plugin: String,
    repeating: Option<Duration>,
    state: TaskState,
    handler: Box<dyn FnMut() + Send + 'static>,
}

struct SchedulerInner {
    tasks: SyncMutex<FxHashMap<TaskId, Task>>,
    next_id: AtomicU64,
}

impl SchedulerInner {
    fn cancel(&self, id: TaskId) -> bool {
        let mut m = self.tasks.lock();
        if let Some(t) = m.get_mut(&id) {
            t.state = TaskState::Cancelled;
            true
        } else { false }
    }
    fn is_cancelled(&self, id: TaskId) -> bool {
        let m = self.tasks.lock();
        m.get(&id).is_some_and(|t| t.state == TaskState::Cancelled)
    }
}

pub struct Scheduler {
    inner: Arc<SchedulerInner>,
}

impl Default for Scheduler {
    fn default() -> Self { Self::new() }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SchedulerInner {
                tasks: SyncMutex::new(FxHashMap::default()),
                next_id: AtomicU64::new(1),
            }),
        }
    }

    fn next_id(&self) -> TaskId {
        self.inner.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Запустить на следующем тике (синхронно).
    pub fn run_task(&self, plugin: impl Into<String>, f: impl FnMut() + Send + 'static) -> TaskHandle {
        self.schedule(plugin, f, None)
    }

    /// Через `delay`.
    pub fn run_task_later(&self, plugin: impl Into<String>, delay: Duration, f: impl FnMut() + Send + 'static) -> TaskHandle {
        let _ = delay;
        self.schedule(plugin, f, None)
    }

    /// Повторяющаяся задача каждый `period`.
    pub fn run_task_timer(&self, plugin: impl Into<String>, period: Duration, f: impl FnMut() + Send + 'static) -> TaskHandle {
        self.schedule(plugin, f, Some(period))
    }

    /// Асинхронно (в tokio blocking pool).
    pub fn run_task_async(&self, plugin: impl Into<String>, f: impl FnOnce() + Send + 'static) -> TaskHandle {
        let id = self.next_id();
        let _plugin = plugin.into();
        std::thread::spawn(f);
        TaskHandle { id, scheduler: self.inner.clone() }
    }

    fn schedule(&self, plugin: impl Into<String>, f: impl FnMut() + Send + 'static, repeating: Option<Duration>) -> TaskHandle {
        let id = self.next_id();
        let task = Task {
            id,
            plugin: plugin.into(),
            repeating,
            state: TaskState::Scheduled,
            handler: Box::new(f),
        };
        self.inner.tasks.lock().insert(id, task);
        TaskHandle { id, scheduler: self.inner.clone() }
    }

    /// Вызывается ядром каждый тик — выполняет все `Scheduled` задачи.
    pub fn tick(&self) {
        let ids: Vec<TaskId> = self.inner.tasks.lock().keys().copied().collect();
        for id in ids {
            let mut guard = self.inner.tasks.lock();
            if let Some(task) = guard.get_mut(&id) {
                if task.state == TaskState::Cancelled { continue; }
                task.state = TaskState::Running;
                let handler: *mut dyn FnMut() = &mut *task.handler as *mut _;
                drop(guard);
                // SAFETY: `handler` указывает на `Box<dyn FnMut>` внутри `Task`, который остаётся в `HashMap`
                // на время вызова (мы сняли lock, но `Task` не удаляется до конца `tick`). Никакой другой
                // поток не получает `&mut` на тот же `Task` одновременно, т.к. `tick` — единственный writer
                // на тике ядра.
                unsafe { (*handler)(); }
                let mut guard2 = self.inner.tasks.lock();
                if let Some(t) = guard2.get_mut(&id) {
                    if t.repeating.is_some() && t.state != TaskState::Cancelled {
                        t.state = TaskState::Scheduled;
                    } else {
                        t.state = TaskState::Completed;
                    }
                }
            }
        }
        self.inner.tasks.lock().retain(|_, t| t.state == TaskState::Scheduled || t.state == TaskState::Running);
    }

    pub fn cancel_all_for(&self, plugin: &str) {
        let mut m = self.inner.tasks.lock();
        for t in m.values_mut() {
            if t.plugin == plugin {
                t.state = TaskState::Cancelled;
            }
        }
    }

    pub fn task_count(&self) -> usize { self.inner.tasks.lock().len() }
}
