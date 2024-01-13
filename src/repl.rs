use hvmc::ast;

#[derive(Debug)]
pub enum Repl {
  Def((String, ast::Net)),
  Net(ast::Net),
  Reload,
  Quit,
}
