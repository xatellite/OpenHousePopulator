mod housenumber;

fn main() {
    let res = housenumber::housenumber_list("17a/27b/12-24/4a/4 a/b/c / 4d").unwrap();
    println!("rest: {}, result: {:?}, pretty: {}", res.0, res.1, res.1);
}
