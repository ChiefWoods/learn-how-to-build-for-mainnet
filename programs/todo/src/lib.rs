use anchor_lang::prelude::*;

declare_id!("EPFBi5maMUcdV3rYj2yssVAhNwAK4nxqjqreML96ubcf");

#[program]
pub mod todo {
    use super::*;

    pub fn save_tasks(ctx: Context<SaveTasks>, replacing_tasks: Vec<Task>) -> Result<()> {
        for task in replacing_tasks.iter() {
            if task.name.len() > 32 {
                return Err(ErrorCode::TaskNameTooLong.into())
            } else if task.name.len() < 1 {
                return Err(ErrorCode::TaskNameTooShort.into())
            }

            for other_task in replacing_tasks.iter() {
                if task.id == other_task.id {
                    return Err(ErrorCode::TaskIdNotUnique.into())
                }
            }
        }

        let tasks = &mut ctx.accounts.tasks;

        if tasks.tasks.len() < replacing_tasks.len() { 
            let new_space = 8 + replacing_tasks.len() * (4 + (4 + 32) + 1);
            let new_minimum_balance = Rent::get()?.minimum_balance(new_space);
            let tasks_account_info = tasks.to_account_info();
            let lamports_diff = new_minimum_balance.saturating_sub(tasks_account_info.lamports());
            **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? -= lamports_diff;
            **tasks_account_info.try_borrow_mut_lamports()? += lamports_diff;
            tasks_account_info.realloc(new_space, false)?;
        }

        tasks.tasks = replacing_tasks;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(replacing_tasks: Vec<Task>)]
pub struct SaveTasks<'info> {
    #[account(
        init_if_needed,
        space = 8 + replacing_tasks.len() * (4 + (4 + 32) + 1),
        payer = user,
        seeds = [user.key().as_ref()],
        bump
    )]
    pub tasks: Account<'info, TasksAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct TasksAccount {
    pub tasks: Vec<Task>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub completed: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Task name is too long")]
    TaskNameTooLong,
    #[msg("Task name is too short")]
    TaskNameTooShort,
    #[msg("Task ID is not unique")]
    TaskIdNotUnique
}