#[macro_use]
extern crate custom_derive;
extern crate clap;

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

#[derive(Debug, Default)]
struct Foo;

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for Foo {
  fn from(_: &'b clap::ArgMatches<'a>) -> Foo {
    Foo::default()
  }
}

impl App for Foo {
  fn app<'a, 'b: 'a>() -> clap::App<'a, 'b> {
    <Self as App>::append(clap::App::new("foo"))
  }
  fn append<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("Foo app")
  }
}


#[derive(Debug, Default)]
struct Bar;

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for Bar {
  fn from(_: &'b clap::ArgMatches<'a>) -> Bar {
    Bar::default()
  }
}

impl App for Bar {
  fn app<'a, 'b: 'a>() -> clap::App<'a, 'b> {
    <Self as App>::append(clap::App::new("bar"))
  }
  fn append<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.about("Bar app")
  }
}


fn main() {
  let ref matches = MyApp::app().get_matches();
  let app: MyApp = matches.into();

  println!("{:?}", app);
}
