pub fn this_should_never_happen<'a, T>(out: &'a Vec<T>, desc: &str) -> Vec<&'a T>{
    // This is to avoid crashes due to what kind of should be a panic, but doesnt need to be
    // (Better to do a weird publish than crash everything for instance)
    println!("This should never have happened: {}", desc);
    out.iter().collect()
}