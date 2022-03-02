fn main() {
    println!("cargo:rerun-if-changed=mysql_migrations");
}