extern crate clap;
#[macro_use]
extern crate clap_derive;

trait App {
  fn app<'a, 'b: 'a>() -> clap::App<'a, 'b>;
  fn append<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b>;
}

#[derive(Debug, App)]
#[clap(name = "myapp", about = "My sample application")]
#[clap(VersionlessSubcommands, SubcommandRequiredElseHelp)]
enum MyApp {
  #[clap(name = "foo")]
  Foo(Foo),
  #[clap(name = "hoge")]
  Bar(Bar),
}

#[derive(Debug, Default, App)]
#[clap(name = "foo", about = "Foo app")]
struct Foo;

#[derive(Debug, Default, App)]
#[clap(name = "bar", about = "Bar app")]
struct Bar;

#[test]
fn test_app() {
  let args = vec!["myapp", "foo"];
  let ref matches = MyApp::app().get_matches_from(args);
  println!("a");
  let app: MyApp = matches.into();

  println!("{:?}", app);
}
