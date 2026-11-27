//! События: шина, приоритеты, отменяемость. 100% покрытие как в Bukkit, но без магии рефлексии.

use std::any::{Any, TypeId};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use rustc_hash::FxHashMap;
use steel_utils::locks::{SyncMutex, SyncRwLock};

/// Любое событие — `'static + Send + Sync`. Имя — для логов/дебага.
pub trait Event: Any + Send + Sync + 'static {
    fn event_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
impl<T: Any + Send + Sync + 'static> Event for T {}

/// Отменяемое событие (аналог `Cancellable` в Bukkit).
pub trait Cancellable {
    fn is_cancelled(&self) -> bool;
    fn set_cancelled(&mut self, cancelled: bool);
}

/// Приоритет — порядок вызова слушателей. `Monitor` — только чтение результата.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventPriority {
    Lowest = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Highest = 4,
    Monitor = 5,
}

impl EventPriority {
    pub const ALL: [EventPriority; 6] = [
        EventPriority::Lowest,
        EventPriority::Low,
        EventPriority::Normal,
        EventPriority::High,
        EventPriority::Highest,
        EventPriority::Monitor,
    ];
}

/// Хэндл подписки — можно вызвать `unsubscribe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerHandle(pub u64);

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

type ArcHandler = Arc<dyn Fn(&mut dyn Any) + Send + Sync>;

struct ListenerEntry {
    id: u64,
    priority: EventPriority,
    handler: ArcHandler,
    #[expect(dead_code, reason = "plugin name stored for debugging, will be used in diagnostics")]
    plugin: String,
}

/// Потокобезопасная шина событий. Плагины подписываются по `TypeId`.
pub struct EventBus {
    listeners: SyncRwLock<FxHashMap<TypeId, Vec<ListenerEntry>>>,

}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: SyncRwLock::new(FxHashMap::default()),
        }
    }

    /// Подписка. Возвращает handle для отписки.
    pub fn subscribe<E, F>(&self, priority: EventPriority, plugin: impl Into<String>, handler: F) -> ListenerHandle
    where
        E: Event,
        F: Fn(&mut E) + Send + Sync + 'static,
    {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let arc: ArcHandler = Arc::new(move |any: &mut dyn Any| {
            if let Some(ev) = any.downcast_mut::<E>() {
                handler(ev);
            }
        });
        let entry = ListenerEntry {
            id,
            priority,
            handler: arc,
            plugin: plugin.into(),
        };
        let mut map = self.listeners.write();
        let vec = map.entry(TypeId::of::<E>()).or_default();
        vec.push(entry);
        vec.sort_by_key(|e| e.priority as u8);
        ListenerHandle(id)
    }

    /// Упрощённая подписка без указания плагина (для тестов/примеров).
    pub fn subscribe_simple<E, F>(&self, priority: EventPriority, handler: F) -> ListenerHandle
    where
        E: Event,
        F: Fn(&mut E) + Send + Sync + 'static,
    {
        self.subscribe::<E, F>(priority, "steel-api", handler)
    }

    pub fn unsubscribe(&self, handle: ListenerHandle) -> bool {
        let mut map = self.listeners.write();
        let mut removed = false;
        for vec in map.values_mut() {
            let before = vec.len();
            vec.retain(|e| e.id != handle.0);
            if vec.len() != before {
                removed = true;
            }
        }
        removed
    }

    /// Вызов события — проход по всем подписчикам по приоритету.
    pub fn fire<E: Event>(&self, event: &mut E) {
        let handlers: Vec<ArcHandler> = {
            let map = self.listeners.read();
            if let Some(vec) = map.get(&TypeId::of::<E>()) {
                vec.iter().map(|e| e.handler.clone()).collect()
            } else {
                Vec::new()
            }
        };
        for h in handlers {
            let any: &mut dyn Any = event as &mut dyn Any;
            h(any);
        }
    }

    pub fn has_listeners<E: Event>(&self) -> bool {
        let map = self.listeners.read();
        map.get(&TypeId::of::<E>()).is_some_and(|v| !v.is_empty())
    }

    pub fn clear(&self) {
        self.listeners.write().clear();
    }

    pub fn listener_count<E: Event>(&self) -> usize {
        let map = self.listeners.read();
        map.get(&TypeId::of::<E>()).map(|v| v.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyEvent {
        pub cancelled: bool,
    }
    impl Cancellable for MyEvent {
        fn is_cancelled(&self) -> bool { self.cancelled }
        fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
    }

    #[test]
    fn priority_order() {
        let bus = EventBus::new();
        let order = Arc::new(SyncMutex::new(Vec::new()));
        let o2 = order.clone();
        bus.subscribe_simple::<MyEvent, _>(EventPriority::Highest, move |_| o2.lock().push(2));
        let o1 = order.clone();
        bus.subscribe_simple::<MyEvent, _>(EventPriority::Lowest, move |_| o1.lock().push(1));
        let o3 = order.clone();
        bus.subscribe_simple::<MyEvent, _>(EventPriority::Normal, move |_| o3.lock().push(3));
        let mut ev = MyEvent { cancelled: false };
        bus.fire(&mut ev);
        assert_eq!(*order.lock(), vec![1, 3, 2]);
    }

    #[test]
    fn cancellable_works() {
        let bus = EventBus::new();
        bus.subscribe_simple::<MyEvent, _>(EventPriority::Normal, |e| e.set_cancelled(true));
        let mut ev = MyEvent { cancelled: false };
        bus.fire(&mut ev);
        assert!(ev.is_cancelled());
    }

    #[test]
    fn unsubscribe_works() {
        let bus = EventBus::new();
        let h = bus.subscribe_simple::<MyEvent, _>(EventPriority::Normal, |_| {});
        assert_eq!(bus.listener_count::<MyEvent>(), 1);
        assert!(bus.unsubscribe(h));
        assert_eq!(bus.listener_count::<MyEvent>(), 0);
    }
}
