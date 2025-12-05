mod build;

use build::parse::PackageParser;

fn main() {
    let parser = PackageParser::new("./tests/package.yaml");
    let package = parser.parse().unwrap_or_else(|e| panic!("{}", e));
    println!("{:#?}", package);
}
