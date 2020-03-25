pub fn get_password() -> Option<String> {
    match rpassword::read_password_from_tty(Some("Password (Just enter for no password): ")) {
        Ok(s) => if s.is_empty() {
            None
        } else {
            Some(s)
        },
        _ => {
            None
        }
    }
}
