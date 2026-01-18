use anchor_lang::prelude::*;

declare_id!("n3qd2EbGLXbVCJhB1H8pQGDTzHfvDJEPqp4PuhZBiz2");

#[program]
pub mod hms_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
