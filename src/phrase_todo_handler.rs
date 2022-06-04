pub(crate) fn handle_phrase_todo(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    println!("dropping todo: {:?}", todo);
    Ok(())
}