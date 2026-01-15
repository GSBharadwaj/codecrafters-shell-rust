
pub struct ShellCmd {
    pub args: Vec<String>,
    pub redirection_path: Option<String>,
    pub redirection_append: bool,
    pub err_redirection_path: Option<String>,
    pub err_redirection_append: bool,
}
