//! Starforge Arena — provably-fair arcade program (Anchor).
//!
//! Flow (mirrors the audited EVM session flow, adapted to Solana slots):
//!   start_session -> reveal_spin -> [submit_picks -> settle_gamble] -> settle
//!
//! Randomness (devnet MVP): the seed is the hash of a future slot committed at
//! `start_session`, read from the SlotHashes sysvar by a permissionless crank.
//! Verifiable by anyone; no oracle, no cost. Mainnet upgrade path: VRF oracle
//! (Orao/Switchboard) behind the `rng` module boundary.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::slot_hashes::SlotHashes;
use starforge_math as m;

declare_id!("9zrUdECfgs7CC2tzgNeRXjEQA6MWvQofFv1Rzbc1oc9m");

/// Slots to wait before the committed randomness becomes available.
pub const REVEAL_DELAY_SLOTS: u64 = 10;
/// Max cascade steps whose grids are stored (matches math crate).
pub const MAX_STEPS: usize = 2;

/// Extract the values `settle_session` needs, ending the `&mut Session` borrow.
/// (macro_rules must be defined before the #[program] module to be in scope)
macro_rules! settle_now {
    ($ctx:expr, $session:expr) => {{
        let wager = $session.wager;
        let total_bps = $session.total_bps;
        let seed = $session.seed;
        let player_key = $session.player;
        let session_key = $ctx.accounts.session.key();
        let session_info = $ctx.accounts.session.to_account_info();
        let player_info = $ctx.accounts.player.to_account_info();
        settle_session(
            &session_info,
            &player_info,
            session_key,
            player_key,
            wager,
            total_bps,
            seed,
        )
    }};
}

#[program]
pub mod starforge {
    use super::*;

    /// Create the House PDA (treasury + config).
    pub fn initialize_house(ctx: Context<InitializeHouse>, fee_bps: u16) -> Result<()> {
        require!(fee_bps <= 1_000, ArenaError::BadFee);
        let house = &mut ctx.accounts.house;
        house.authority = ctx.accounts.authority.key();
        house.fee_bps = fee_bps;
        house.bump = ctx.bumps.house;
        Ok(())
    }

    /// Open a session: player locks `wager` lamports in the Session PDA vault
    /// and commits to randomness from a future slot.
    pub fn start_session(
        ctx: Context<StartSession>,
        session_id: u64,
        wager: u64,
        artifacts: u8,
    ) -> Result<()> {
        require!(wager > 0, ArenaError::BadWager);
        require!(artifacts <= 0x07, ArenaError::BadArtifacts);

        // Move wager into the session vault (PDA-owned system account).
        let cpi = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.player.to_account_info(),
                to: ctx.accounts.session.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi, wager)?;

        let clock = Clock::get()?;
        let session = &mut ctx.accounts.session;
        session.player = ctx.accounts.player.key();
        session.session_id = session_id;
        session.wager = wager;
        session.artifacts = artifacts;
        session.stage = m::STAGE_SPIN;
        session.request_slot = clock.slot + REVEAL_DELAY_SLOTS;
        session.bump = ctx.bumps.session;
        Ok(())
    }

    /// Permissionless crank: after `request_slot`, read the slot hash as seed
    /// and run the full spin math. Supernova (4+ stars) -> PICKS, else settle.
    pub fn reveal_spin(ctx: Context<RevealSpin>) -> Result<()> {
        let session = &mut ctx.accounts.session;
        require!(session.stage == m::STAGE_SPIN, ArenaError::WrongStage);

        let clock = Clock::get()?;
        require!(
            clock.slot > session.request_slot,
            ArenaError::RandomnessNotReady
        );
        let slot_hashes = SlotHashes::from_account_info(&ctx.accounts.slot_hashes)?;
        let entry = slot_hashes
            .get(&session.request_slot)
            .ok_or(ArenaError::SlotHashExpired)?;
        let seed: [u8; 32] = entry.to_bytes();

        let outcome = m::run_spin(seed, session.artifacts);
        session.grid_win_bps = outcome.grid_win_bps;
        session.stars = outcome.stars;
        session.step_count = outcome.grids.len() as u8;
        for (i, g) in outcome.grids.iter().enumerate() {
            session.grids[i * m::CELLS..(i + 1) * m::CELLS].copy_from_slice(g);
        }
        for (i, &p) in outcome.step_pats.iter().enumerate() {
            session.step_pats[i] = p;
        }
        for (i, &w) in outcome.step_wins_bps.iter().enumerate() {
            session.step_wins_bps[i] = w;
        }
        session.seed = seed;

        if let Some(prizes) = outcome.prizes {
            session.prizes = prizes;
            session.stage = m::STAGE_PICKS;
        } else {
            session.total_bps = outcome.grid_win_bps;
            session.stage = m::STAGE_DONE;
        }

        emit!(SpinRevealed {
            session: session.key(),
            player: session.player,
            seed,
            supernova: outcome.prizes.is_some(),
            grid_win_bps: outcome.grid_win_bps,
        });

        if session.stage == m::STAGE_DONE {
            settle_now!(ctx, session)?;
        }
        Ok(())
    }

    /// Player submits 5 distinct picks (0..11) + gamble flag.
    pub fn submit_picks(ctx: Context<PlayerAction>, picks: [u8; 5], gamble: bool) -> Result<()> {
        let session = &mut ctx.accounts.session;
        require!(session.stage == m::STAGE_PICKS, ArenaError::WrongStage);
        require!(m::valid_picks(&picks), ArenaError::BadPicks);

        session.picks = picks;
        session.picks_bps = m::picks_bps(&session.prizes, &picks, session.artifacts);

        if gamble {
            let clock = Clock::get()?;
            session.stage = m::STAGE_GAMBLE;
            session.request_slot = clock.slot + REVEAL_DELAY_SLOTS;
        } else {
            session.total_bps = session.grid_win_bps + session.picks_bps;
            session.stage = m::STAGE_DONE;
            settle_now!(ctx, session)?;
        }
        Ok(())
    }

    /// Permissionless crank: fair coin from a fresh future-slot seed.
    pub fn settle_gamble(ctx: Context<RevealSpin>) -> Result<()> {
        let session = &mut ctx.accounts.session;
        require!(session.stage == m::STAGE_GAMBLE, ArenaError::WrongStage);

        let clock = Clock::get()?;
        require!(
            clock.slot > session.request_slot,
            ArenaError::RandomnessNotReady
        );
        let slot_hashes = SlotHashes::from_account_info(&ctx.accounts.slot_hashes)?;
        let entry = slot_hashes
            .get(&session.request_slot)
            .ok_or(ArenaError::SlotHashExpired)?;
        let seed: [u8; 32] = entry.to_bytes();

        session.gamble_win = m::coin_flip(seed);
        session.total_bps = session.grid_win_bps
            + if session.gamble_win {
                session.picks_bps * 2
            } else {
                0
            };
        session.stage = m::STAGE_DONE;
        settle_now!(ctx, session)?;
        Ok(())
    }
}

// ---------------- settlement (single payout path) ----------------

fn settle_session(
    session_info: &AccountInfo,
    player_info: &AccountInfo,
    session_key: Pubkey,
    player: Pubkey,
    wager: u64,
    total_bps: u64,
    seed: [u8; 32],
) -> Result<()> {
    let payout = m::payout_for(wager, total_bps);

    // Pay the player from the session vault (lamport transfer, no CPI needed
    // for system-owned accounts).
    let vault_lamports = session_info.lamports();
    require!(vault_lamports >= payout, ArenaError::InsufficientVault);
    **session_info.try_borrow_mut_lamports()? -= payout;
    **player_info.try_borrow_mut_lamports()? += payout;

    emit!(SessionSettled {
        session: session_key,
        player,
        wager,
        total_bps,
        payout,
        seed,
    });
    // NOTE: full close (rent reclaim) is done by the client via a follow-up
    // `close_session` instruction in the production build; kept out of the
    // scaffold to keep the account-borrow story simple for review.
    Ok(())
}

// ---------------- accounts ----------------

#[account]
pub struct House {
    pub authority: Pubkey,
    pub fee_bps: u16,
    pub bump: u8,
}

#[account]
pub struct Session {
    pub player: Pubkey,
    pub session_id: u64,
    pub wager: u64,
    pub artifacts: u8,
    pub stage: u8,
    pub request_slot: u64,
    pub seed: [u8; 32],
    pub grid_win_bps: u64,
    pub picks_bps: u64,
    pub total_bps: u64,
    pub stars: u8,
    pub prizes: [u16; 12],
    pub picks: [u8; 5],
    pub gamble_win: bool,
    /// Pre-refill grids, MAX_STEPS * 30 bytes (column-major).
    pub grids: [u8; MAX_STEPS * m::CELLS],
    pub step_pats: [u8; MAX_STEPS],
    /// Payout (lamports... no: bps of wager) per cascade step, for the animated reveal.
    pub step_wins_bps: [u64; MAX_STEPS],
    pub step_count: u8,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct InitializeHouse<'info> {
    #[account(
        init, payer = authority,
        space = 8 + 32 + 2 + 1,
        seeds = [b"house"], bump
    )]
    pub house: Account<'info, House>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(session_id: u64)]
pub struct StartSession<'info> {
    #[account(
        init, payer = player,
        space = 8 + 32 + 8 + 8 + 1 + 1 + 8 + 32 + 8 + 8 + 8 + 1 + 24 + 5 + 1 + 60 + 2 + 16 + 1 + 1,
        seeds = [b"session", player.key().as_ref(), &session_id.to_le_bytes()],
        bump
    )]
    pub session: Account<'info, Session>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevealSpin<'info> {
    #[account(mut)]
    pub session: Account<'info, Session>,
    /// CHECK: must equal session.player; credited with the payout on settle.
    #[account(mut, constraint = player.key() == session.player @ ArenaError::UnauthorizedPlayer)]
    pub player: AccountInfo<'info>,
    /// CHECK: SlotHashes sysvar, read-only.
    pub slot_hashes: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct PlayerAction<'info> {
    #[account(mut, has_one = player)]
    pub session: Account<'info, Session>,
    pub player: Signer<'info>,
}

// ---------------- events & errors ----------------

#[event]
pub struct SpinRevealed {
    pub session: Pubkey,
    pub player: Pubkey,
    pub seed: [u8; 32],
    pub supernova: bool,
    pub grid_win_bps: u64,
}

#[event]
pub struct SessionSettled {
    pub session: Pubkey,
    pub player: Pubkey,
    pub wager: u64,
    pub total_bps: u64,
    pub payout: u64,
    pub seed: [u8; 32],
}

#[error_code]
pub enum ArenaError {
    #[msg("wager must be > 0")]
    BadWager,
    #[msg("artifacts bitmask must be <= 0x07")]
    BadArtifacts,
    #[msg("picks must be 5 distinct indices in [0,12)")]
    BadPicks,
    #[msg("instruction called in the wrong stage")]
    WrongStage,
    #[msg("randomness slot not reached yet")]
    RandomnessNotReady,
    #[msg("slot hash expired from sysvar (512-slot window)")]
    SlotHashExpired,
    #[msg("fee_bps must be <= 1000")]
    BadFee,
    #[msg("session vault cannot cover payout")]
    InsufficientVault,
    #[msg("player account does not match the session owner")]
    UnauthorizedPlayer,
}
