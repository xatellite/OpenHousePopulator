mod housenumber;

fn main() {
    let res = housenumber::housenumber_list("17a/27b");
    println!("{:?}", res);
}