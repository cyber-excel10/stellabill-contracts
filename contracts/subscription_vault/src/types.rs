//! Contract types: errors, state, and events.
//!
//! Kept in a separate module to reduce merge conflicts when editing state machine
//! or contract entrypoints.

use soroban_sdk::{contracterror, contracttype, Address};

pub const BILLING_SNAPSHOT_FLAG_CLOSED: u32 = 1 << 0;
pub const BILLING_SNAPSHOT_FLAG_INTERVAL_CHARGED: u32 = 1 << 1;
pub const BILLING_SNAPSHOT_FLAG_USAGE_CHARGED: u32 = 1 << 2;
pub const BILLING_SNAPSHOT_FLAG_EMPTY_PERIOD: u32 = 1 << 3;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    MerchantSubs(Address),
    MerchantPaused(Address),
    EmergencyStop,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingPeriodSnapshot {
    pub subscription_id: u32,
    pub period_index: u32,
    pub period_start_timestamp: u64,
    pub period_end_timestamp: u64,
    pub total_amount_charged: i128,
    pub total_usage_units: i128,
    pub status_flags: u32,
    EmergencyStop,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    // --- Auth Errors (401-403) ---
    /// Caller does not have the required authorization.
    Unauthorized = 401,
    /// Caller is authorized but does not have permission for this specific action.
    Forbidden = 403,

    // --- Not Found (404) ---
    /// The requested resource was not found in storage.
    NotFound = 404,

    // --- Invalid Input (400, 402, 405-410) ---
    /// The requested state transition is not allowed by the state machine.
    InvalidStatusTransition = 400,
    /// The top-up amount is below the minimum required threshold.
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    InvalidStatusTransition = 400,
    BelowMinimumTopup = 402,
    InvalidRecoveryAmount = 1008,
    SubscriptionExpired = 410,
    SubscriptionLimitReached = 429,

    // --- Operational Errors (1001+) ---
    IntervalNotElapsed = 1001,
    NotActive = 1002,
    InsufficientBalance = 1003,
    UsageNotEnabled = 1004,
    InsufficientPrepaidBalance = 1005,
    InvalidAmount = 1006,
    Replay = 1007,
    EmergencyStopActive = 1009,
    Underflow = 1010,
    RecoveryNotAllowed = 1011,
    Overflow = 1012,
    NotInitialized = 1013,
    InvalidExportLimit = 1014,
    InvalidInput = 1015,
    Reentrancy = 1016,
    /// Lifetime charge cap has been reached; no further charges are allowed.
    LifetimeCapReached = 1017,
    AlreadyInitialized = 1018,
    UsageCapExceeded = 1019,
    RateLimitExceeded = 1020,
    InvalidFeeBps = 1021,
    TreasuryNotConfigured = 1022,
    MerchantPaused = 1023,
    SubscriberBlocklisted = 1023,
}

impl Error {
    pub const fn to_code(self) -> u32 {
        self as u32
    }
}

/// Result of charging one subscription in a batch.
#[contracttype]
#[derive(Clone, Debug)]
pub struct BatchChargeResult {
    pub success: bool,
    /// If success is false, the error code; otherwise 0.
    pub error_code: u32,
}

/// Result of a batch merchant withdrawal operation.
#[contracttype]
#[derive(Clone, Debug)]
pub struct BatchWithdrawResult {
    pub success: bool,
    pub error_code: u32,
}

/// Represents the lifecycle state of a subscription.
///
/// See `docs/subscription_lifecycle.md` for how each status is entered and exited.
///
/// # State Machine
///
/// - **Active**: Subscription is active and charges can be processed.
///   - Can transition to: `Paused`, `Cancelled`, `InsufficientBalance`, `GracePeriod`
/// - **Paused**: Subscription is temporarily suspended, no charges processed.
///   - Can transition to: `Active`, `Cancelled`
/// - **Cancelled**: Subscription is permanently terminated (terminal state).
///   - No outgoing transitions
/// - **InsufficientBalance**: Subscription failed due to insufficient funds.
///   - Can transition to: `Active` (after deposit + resume), `Cancelled`
/// - **GracePeriod**: Subscription is in grace period after a missed charge.
///   - Can transition to: `Active`, `InsufficientBalance`, `Cancelled`
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    Active = 0,
    Paused = 1,
    Cancelled = 2,
    InsufficientBalance = 3,
    GracePeriod = 4,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Subscription {
    pub subscriber: Address,
    pub merchant: Address,
    pub amount: i128,
    pub interval_seconds: u64,
    pub last_payment_timestamp: u64,
    pub status: SubscriptionStatus,
    pub prepaid_balance: i128,
    pub usage_enabled: bool,
    pub expiration: Option<u64>,
    pub billing_anchor_timestamp: u64,
    pub current_period_index: u32,
    pub current_period_usage_units: i128,
    pub usage_cap_units: Option<i128>,
    pub usage_rate_limit_max_calls: Option<u32>,
    pub usage_rate_window_secs: u64,
    pub lifetime_cap: Option<i128>,
    pub lifetime_charged: i128,
    /// Optional expiration timestamp. Subscription cannot be charged after this time.
    pub expiration: Option<u64>,
    /// Billing anchor timestamp - the reference point for calculating billing periods.
    pub billing_anchor_timestamp: u64,
    /// Current billing period index.
    pub current_period_index: u32,
    /// Usage units consumed in the current period.
    pub current_period_usage_units: i128,
    /// Optional usage cap per billing period.
    pub usage_cap_units: Option<i128>,
    /// Optional rate limit: max calls per window.
    pub usage_rate_limit_max_calls: Option<u32>,
    /// Rate limit window in seconds.
    pub usage_rate_window_secs: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingPeriodSnapshot {
    pub subscription_id: u32,
    pub period_index: u32,
    pub period_start_timestamp: u64,
    pub period_end_timestamp: u64,
    pub total_amount_charged: i128,
    pub total_usage_units: i128,
    pub status_flags: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BatchChargeResult {
    pub success: bool,
    pub error_code: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BatchWithdrawResult {
    pub success: bool,
    pub error_code: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractSnapshot {
    pub admin: Address,
    pub token: Address,
    pub min_topup: i128,
    pub next_id: u32,
    pub storage_version: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct SubscriptionSummary {
    pub subscription_id: u32,
    pub subscriber: Address,
    pub merchant: Address,
    pub amount: i128,
    pub interval_seconds: u64,
    pub last_payment_timestamp: u64,
    pub status: SubscriptionStatus,
    pub prepaid_balance: i128,
    pub usage_enabled: bool,
    pub lifetime_cap: Option<i128>,
    pub lifetime_charged: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MigrationExportEvent {
    pub admin: Address,
    pub start_id: u32,
    pub limit: u32,
    pub exported: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlanTemplate {
    pub merchant: Address,
    pub amount: i128,
    pub interval_seconds: u64,
    pub usage_enabled: bool,
    pub lifetime_cap: Option<i128>,
}

/// Result of computing next charge information for a subscription.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NextChargeInfo {
    /// Estimated timestamp for the next charge attempt.
    pub next_charge_timestamp: u64,
    /// Whether a charge is actually expected based on the subscription status.
    pub is_charge_expected: bool,
}

/// View of a subscription's lifetime cap status.
///
/// Returned by `get_cap_info` for off-chain dashboards and UX displays.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NextChargeInfo {
    pub next_charge_timestamp: u64,
    pub is_charge_expected: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapInfo {
    pub lifetime_cap: Option<i128>,
    pub lifetime_charged: i128,
    pub remaining_cap: Option<i128>,
    pub cap_reached: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct EmergencyStopEnabledEvent {
    pub admin: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct EmergencyStopDisabledEvent {
    pub admin: Address,
    pub timestamp: u64,
}

/// Event emitted when a merchant-wide pause is enabled.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MerchantPausedEvent {
    pub merchant: Address,
    pub timestamp: u64,
}

/// Event emitted when a merchant-wide pause is disabled.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MerchantUnpausedEvent {
    pub merchant: Address,
    pub timestamp: u64,
}

/// Represents the reason for stranded funds that can be recovered by admin.

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryReason {
    AccidentalTransfer = 0,
    DeprecatedFlow = 1,
    UnreachableSubscriber = 2,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RecoveryEvent {
    pub admin: Address,
    pub recipient: Address,
    pub amount: i128,
    pub reason: RecoveryReason,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct UsageCapReachedEvent {
    pub subscription_id: u32,
    pub period_index: u32,
    pub cap_units: i128,
    pub attempted_units: i128,
}

/// Event emitted when protocol fee is skimmed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProtocolFeeSkimmedEvent {
    pub subscription_id: u32,
    pub merchant: Address,
    pub treasury: Address,
    pub gross_amount: i128,
    pub fee_amount: i128,
    pub net_amount: i128,
}

/// Event emitted when a subscription is created.

#[contracttype]
#[derive(Clone, Debug)]
pub struct SubscriptionCreatedEvent {
    pub subscription_id: u32,
    pub subscriber: Address,
    pub merchant: Address,
    pub amount: i128,
    pub interval_seconds: u64,
}

/// Event emitted when funds are deposited into a subscription vault.

    pub lifetime_cap: Option<i128>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct FundsDepositedEvent {
    pub subscription_id: u32,
    pub subscriber: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct SubscriptionChargedEvent {
    pub subscription_id: u32,
    pub merchant: Address,
    pub amount: i128,
    pub lifetime_charged: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct SubscriptionCancelledEvent {
    pub subscription_id: u32,
    pub authorizer: Address,
    pub refund_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct SubscriptionPausedEvent {
    pub subscription_id: u32,
    pub authorizer: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct SubscriptionResumedEvent {
    pub subscription_id: u32,
    pub authorizer: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MerchantWithdrawalEvent {
    pub merchant: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct OneOffChargedEvent {
    pub subscription_id: u32,
    pub merchant: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct LifetimeCapReachedEvent {
    pub subscription_id: u32,
    pub lifetime_cap: i128,
    pub lifetime_charged: i128,
    pub timestamp: u64,
}
