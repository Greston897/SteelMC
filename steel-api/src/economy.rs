//! Экономика-API — полный аналог Vault `Economy` + Essentials `Economy`.
//! Плагин регистрирует провайдер, остальные плагины вызывают `economy().deposit(...)`.

use rustc_hash::FxHashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EconomyError {
    #[error("недостаточно средств: нужно {need}, есть {have}")]
    Insufficient { need: f64, have: f64 },
    #[error("игрок не найден")]
    PlayerNotFound,
    #[error("{0}")]
    Other(String),
}

pub type EconomyResult<T> = Result<T, EconomyError>;

/// Ответ транзакции — как в Vault `EconomyResponse`.
#[derive(Debug, Clone)]
pub struct EconomyResponse {
    pub amount: f64,
    pub balance: f64,
    pub success: bool,
    pub error: Option<String>,
}

impl EconomyResponse {
    pub fn success(amount: f64, balance: f64) -> Self { Self { amount, balance, success: true, error: None } }
    pub fn failure(amount: f64, balance: f64, err: impl Into<String>) -> Self { Self { amount, balance, success: false, error: Some(err.into()) } }
}

/// Провайдер экономики — реализует любой плагин (Essentials-like).
pub trait EconomyProvider: Send + Sync + 'static {
    fn name(&self) -> &str { "steel_economy" }
    fn has_account(&self, player: uuid::Uuid) -> bool;
    fn create_account(&self, player: uuid::Uuid) -> EconomyResult<()>;
    fn balance(&self, player: uuid::Uuid) -> EconomyResult<f64>;
    fn has(&self, player: uuid::Uuid, amount: f64) -> EconomyResult<bool> { Ok(self.balance(player)? >= amount) }
    fn withdraw(&self, player: uuid::Uuid, amount: f64) -> EconomyResult<EconomyResponse>;
    fn deposit(&self, player: uuid::Uuid, amount: f64) -> EconomyResult<EconomyResponse>;
    fn transfer(&self, from: uuid::Uuid, to: uuid::Uuid, amount: f64) -> EconomyResult<EconomyResponse> {
        let w=self.withdraw(from, amount)?;
        if !w.success { return Ok(w); }
        let d=self.deposit(to, amount)?;
        if !d.success { let _=self.deposit(from, amount); }
        Ok(d)
    }
    fn format(&self, amount: f64) -> String { format!("{amount:.2} coins") }
    fn currency_name_singular(&self) -> &str { "coin" }
    fn currency_name_plural(&self) -> &str { "coins" }

    // — банк (Vault Bank) —
    fn create_bank(&self, _name: &str, _owner: uuid::Uuid) -> EconomyResult<()> { Err(EconomyError::Other("banks not supported".into())) }
    fn bank_balance(&self, _name: &str) -> EconomyResult<f64> { Err(EconomyError::Other("banks not supported".into())) }
    fn bank_deposit(&self, _name: &str, _amount: f64) -> EconomyResult<EconomyResponse> { Err(EconomyError::Other("banks not supported".into())) }
    fn bank_withdraw(&self, _name: &str, _amount: f64) -> EconomyResult<EconomyResponse> { Err(EconomyError::Other("banks not supported".into())) }
    fn bank_has(&self, _name: &str, _amount: f64) -> EconomyResult<bool> { Ok(false) }
    fn is_bank_owner(&self, _name: &str, _player: uuid::Uuid) -> EconomyResult<bool> { Ok(false) }
    fn is_bank_member(&self, _name: &str, _player: uuid::Uuid) -> EconomyResult<bool> { Ok(false) }
    fn list_banks(&self) -> Vec<String> { Vec::new() }
}

/// Хэндл экономики — инжектится в `PluginContext`.
pub struct EconomyHandle {
    provider: steel_utils::locks::SyncRwLock<Option<std::sync::Arc<dyn EconomyProvider>>>,
}

impl Default for EconomyHandle { fn default() -> Self { Self::new() } }

impl EconomyHandle {
    pub fn new() -> Self { Self { provider: steel_utils::locks::SyncRwLock::new(None) } }
    pub fn set_provider(&self, p: std::sync::Arc<dyn EconomyProvider>) { *self.provider.write() = Some(p); }
    pub fn provider(&self) -> Option<std::sync::Arc<dyn EconomyProvider>> { self.provider.read().clone() }
    pub fn has_provider(&self) -> bool { self.provider.read().is_some() }
    pub fn balance(&self, player: uuid::Uuid) -> EconomyResult<f64> {
        self.provider.read().as_ref().ok_or(EconomyError::Other("no economy provider".into()))?.balance(player)
    }
    pub fn deposit(&self, player: uuid::Uuid, amount: f64) -> EconomyResult<EconomyResponse> {
        self.provider.read().as_ref().ok_or(EconomyError::Other("no provider".into()))?.deposit(player, amount)
    }
    pub fn withdraw(&self, player: uuid::Uuid, amount: f64) -> EconomyResult<EconomyResponse> {
        self.provider.read().as_ref().ok_or(EconomyError::Other("no provider".into()))?.withdraw(player, amount)
    }
}

/// Заглушка — in-memory для тестов.
pub struct InMemoryEconomy {
    balances: steel_utils::locks::SyncMutex<rustc_hash::FxHashMap<uuid::Uuid, f64>>,
}

impl InMemoryEconomy {
    pub fn new() -> Self { Self { balances: steel_utils::locks::SyncMutex::new(Default::default()) } }
}

impl EconomyProvider for InMemoryEconomy {
    fn has_account(&self, p: uuid::Uuid) -> bool { self.balances.lock().contains_key(&p) }
    fn create_account(&self, p: uuid::Uuid) -> EconomyResult<()> { self.balances.lock().entry(p).or_insert(0.0); Ok(()) }
    fn balance(&self, p: uuid::Uuid) -> EconomyResult<f64> { Ok(*self.balances.lock().get(&p).unwrap_or(&0.0)) }
    fn withdraw(&self, p: uuid::Uuid, a: f64) -> EconomyResult<EconomyResponse> {
        let mut m=self.balances.lock();
        let bal=*m.get(&p).unwrap_or(&0.0);
        if bal < a { return Err(EconomyError::Insufficient { need: a, have: bal }); }
        let nb=bal-a; m.insert(p, nb); Ok(EconomyResponse::success(a, nb))
    }
    fn deposit(&self, p: uuid::Uuid, a: f64) -> EconomyResult<EconomyResponse> {
        let mut m=self.balances.lock();
        let bal=*m.get(&p).unwrap_or(&0.0);
        let nb=bal+a; m.insert(p, nb); Ok(EconomyResponse::success(a, nb))
    }
}
