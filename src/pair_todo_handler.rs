use crate::models::Todo;

pub fn handle_pair_todo(todo: Todo) -> Result<(), anyhow::Error> {
    println!("dropping pair todo");
    Ok(())
}
