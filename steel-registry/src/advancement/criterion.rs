use steel_utils::Identifier;

/// A criterion generated from the vanilla data pack.
///
/// Vanilla pairs every criterion with a typed trigger and a predicate parsed from
/// the JSON. Steel keeps the trigger id and the raw instance so the server can
/// match a fired trigger against every advancement at once; predicate evaluation
/// inside a single criterion is not modelled yet.
pub struct SimpleCriterion {
    trigger: Identifier,
    instance: String,
}

impl SimpleCriterion {
    /// Creates a criterion for `trigger` from its raw JSON instance.
    #[must_use]
    pub const fn new(trigger: Identifier, instance: String) -> Self {
        Self { trigger, instance }
    }
}

impl CriterionTriggerInstance for SimpleCriterion {}

impl SimpleCriterion {
    /// Returns the raw JSON instance vanilla stores next to the trigger.
    #[must_use]
    pub fn instance(&self) -> &str {
        &self.instance
    }
}

impl AnyCriterion for SimpleCriterion {
    fn id(&self) -> Identifier {
        self.trigger.clone()
    }

    fn instance(&self) -> Option<&str> {
        Some(&self.instance)
    }
}

pub struct ImpossibleInstance;
impl CriterionTriggerInstance for ImpossibleInstance {}

pub struct ImpossibleTrigger;
impl CriterionTrigger<ImpossibleInstance> for ImpossibleTrigger {
    fn id(&self) -> Identifier {
        Identifier::vanilla_static("impossible")
    }

    fn parse_instance(&self, _instance: String) -> ImpossibleInstance {
        ImpossibleInstance
    }
}

pub trait AnyCriterion: Send + Sync {
    fn id(&self) -> Identifier;

    /// Returns the raw JSON instance, when the criterion was generated from data.
    ///
    /// Vanilla parses this into a typed predicate; Steel keeps it verbatim and
    /// evaluates it lazily against the event that fired the trigger.
    fn instance(&self) -> Option<&str> {
        None
    }
}

impl<T, G> AnyCriterion for Criterion<T, G>
where
    T: CriterionTriggerInstance + 'static,
    G: CriterionTrigger<T> + 'static,
{
    fn id(&self) -> Identifier {
        self.trigger.id()
    }
}

#[expect(dead_code)]
pub struct Criterion<
    T: CriterionTriggerInstance + 'static,
    G: CriterionTrigger<T> + ?Sized + 'static,
> {
    trigger: &'static G,
    instance: Box<T>,
}

static IMPOSSIBLE_TRIGGER: ImpossibleTrigger = ImpossibleTrigger;

impl<T: CriterionTriggerInstance, G: CriterionTrigger<T> + ?Sized> Criterion<T, G> {
    pub fn new(trigger: &'static G, instance: T) -> Self {
        Self {
            trigger,
            instance: Box::new(instance),
        }
    }
}

impl Default for Criterion<ImpossibleInstance, ImpossibleTrigger> {
    fn default() -> Self {
        Criterion {
            trigger: &IMPOSSIBLE_TRIGGER,
            instance: Box::new(ImpossibleInstance),
        }
    }
}

pub trait CriterionTrigger<T>: Send + Sync
where
    T: CriterionTriggerInstance,
{
    fn id(&self) -> Identifier;
    fn parse_instance(&self, instance: String) -> T;
}

pub trait CriterionTriggerInstance: Send + Sync {}
