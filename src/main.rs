mod repl;

use std::io::Write;

use hvmc::ast::name_to_val;
use lalrpop_util::lalrpop_mod;
use repl::Repl;

lalrpop_mod!(parser);

fn main() {
  if let Err(e) = run() {
    eprintln!("{e}");
  }
}

fn run() -> Result<(), String> {
  let args = std::env::args().collect::<Vec<String>>();

  let file_name = args.get(1).expect("To have a file argument.");
  let Ok(file) = std::fs::read_to_string(file_name) else {
    return Err(format!("Could not read the file '{file_name}'."));
  };

  let book_parser = parser::BookParser::new();
  let ast_book = book_parser.parse(&file).expect("To parse book.");
  let mut rt_book = hvmc::ast::book_to_runtime(&ast_book);

  loop {
    let mut input = String::new();
    std::io::stdout().write(b"> ").unwrap();
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut input).unwrap();

    let parser = parser::ReplParser::new();
    match parser.parse(&input) {
      Ok(rpl) => match rpl {
        Repl::Def((nam, net)) => {
          let nam_val = name_to_val(&nam);
          let data = hvmc::run::Heap::init(1 << 16);
          let mut rt_net = hvmc::run::Net::new(&data);
          hvmc::ast::net_to_runtime(&mut rt_net, &net);
          rt_book.def(nam_val, hvmc::ast::runtime_net_to_runtime_def(&rt_net));
        }
        Repl::Net(net) => {
          let def_data = hvmc::run::Heap::init(1 << 16);
          let mut rt = hvmc::run::Net::new(&def_data);
          hvmc::ast::net_to_runtime(&mut rt, &net);
          rt_book.def(0, hvmc::ast::runtime_net_to_runtime_def(&rt));

          let data = hvmc::run::Heap::init(1 << 16);
          let mut net = hvmc::run::Net::new(&data);
          net.boot(0);
          net.parallel_normal(&rt_book);
          println!("{}", hvmc::ast::show_runtime_net(&net));
        }
        Repl::Reload => {
          let file = &mut std::fs::read_to_string(&file_name).unwrap();
          let ast_book = book_parser.parse(&file).expect("To parse book.");
          let rt_book_ = hvmc::ast::book_to_runtime(&ast_book);
          rt_book.defs.extend(rt_book_.defs);
          println!("Ok.");
        }
        Repl::Quit => break,
      },
      Err(e) => eprintln!("{e}"),
    }
  }
  Ok(())
}
