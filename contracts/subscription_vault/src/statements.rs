//! Billing statement append-only storage and paginated views.

use crate::types::{BillingChargeKind, BillingStatement, BillingStatementsPage, Error};
use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

const KEY_STATEMENT_NEXT: Symbol = symbol_short!("snext");
const KEY_STATEMENT_ROW: Symbol = symbol_short!("srow");

fn next_statement_key(subscription_id: u32) -> (Symbol, u32) {
    (KEY_STATEMENT_NEXT, subscription_id)
}

fn statement_row_key(subscription_id: u32, sequence: u32) -> (Symbol, u32, u32) {
    (KEY_STATEMENT_ROW, subscription_id, sequence)
}

pub fn append_statement(
    env: &Env,
    subscription_id: u32,
    amount: i128,
    merchant: Address,
    kind: BillingChargeKind,
    period_start: u64,
    period_end: u64,
) {
    let storage = env.storage().instance();
    let next: u32 = storage.get(&next_statement_key(subscription_id)).unwrap_or(0);
    let statement = BillingStatement {
        subscription_id,
        sequence: next,
        charged_at: env.ledger().timestamp(),
        period_start,
        period_end,
        amount,
        merchant,
        kind,
    };
    storage.set(&statement_row_key(subscription_id, next), &statement);
    storage.set(&next_statement_key(subscription_id), &(next + 1));
}

pub fn get_total_statements(env: &Env, subscription_id: u32) -> u32 {
    env.storage()
        .instance()
        .get(&next_statement_key(subscription_id))
        .unwrap_or(0)
}

/// Offset/limit pagination over immutable statements.
///
/// When `newest_first` is true, offset 0 returns most recent statement.
pub fn get_statements_by_subscription_offset(
    env: &Env,
    subscription_id: u32,
    offset: u32,
    limit: u32,
    newest_first: bool,
) -> Result<BillingStatementsPage, Error> {
    if limit == 0 {
        return Err(Error::InvalidInput);
    }

    let total = get_total_statements(env, subscription_id);
    if total == 0 || offset >= total {
        return Ok(BillingStatementsPage {
            statements: Vec::new(env),
            next_cursor: None,
            total,
        });
    }

    let mut out = Vec::new(env);
    let remaining = total - offset;
    let page_len = if limit > remaining { remaining } else { limit };
    let mut i = 0u32;
    while i < page_len {
        let seq = if newest_first {
            total - 1 - (offset + i)
        } else {
            offset + i
        };
        if let Some(row) = env
            .storage()
            .instance()
            .get::<_, BillingStatement>(&statement_row_key(subscription_id, seq))
        {
            out.push_back(row);
        }
        i += 1;
    }

    let consumed = offset + page_len;
    let next_cursor = if consumed < total {
        if newest_first {
            Some(total - 1 - consumed)
        } else {
            Some(consumed)
        }
    } else {
        None
    };

    Ok(BillingStatementsPage {
        statements: out,
        next_cursor,
        total,
    })
}

/// Cursor pagination.
///
/// `cursor` represents the sequence index to start from (inclusive).
/// If `None`, starts from newest or oldest depending on `newest_first`.
pub fn get_statements_by_subscription_cursor(
    env: &Env,
    subscription_id: u32,
    cursor: Option<u32>,
    limit: u32,
    newest_first: bool,
) -> Result<BillingStatementsPage, Error> {
    if limit == 0 {
        return Err(Error::InvalidInput);
    }

    let total = get_total_statements(env, subscription_id);
    if total == 0 {
        return Ok(BillingStatementsPage {
            statements: Vec::new(env),
            next_cursor: None,
            total,
        });
    }

    let start = match cursor {
        Some(c) => c,
        None => {
            if newest_first {
                total - 1
            } else {
                0
            }
        }
    };

    if start >= total {
        return Ok(BillingStatementsPage {
            statements: Vec::new(env),
            next_cursor: None,
            total,
        });
    }

    let mut out = Vec::new(env);
    let mut taken = 0u32;

    if newest_first {
        let mut seq = start;
        loop {
            if taken >= limit {
                break;
            }
            if let Some(row) = env
                .storage()
                .instance()
                .get::<_, BillingStatement>(&statement_row_key(subscription_id, seq))
            {
                out.push_back(row);
                taken += 1;
            }
            if seq == 0 {
                break;
            }
            seq -= 1;
        }
        let next_cursor = if start + 1 > taken {
            Some(start - taken)
        } else {
            None
        };
        return Ok(BillingStatementsPage {
            statements: out,
            next_cursor,
            total,
        });
    }

    let mut seq = start;
    while seq < total && taken < limit {
        if let Some(row) = env
            .storage()
            .instance()
            .get::<_, BillingStatement>(&statement_row_key(subscription_id, seq))
        {
            out.push_back(row);
            taken += 1;
        }
        seq += 1;
    }
    let next_cursor = if seq < total { Some(seq) } else { None };

    Ok(BillingStatementsPage {
        statements: out,
        next_cursor,
        total,
    })
}
