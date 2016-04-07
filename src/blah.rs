let path = Path::new("/Users/xavier/code/rust/sen/missile1.h");
let display = path.display();

let mut file = match File::open(&path) {
    Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
    Ok(file) => file,
};

let mut buffer = Vec::new();

match file.read_to_end(&mut buffer) {
    Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
    Ok(_) => println!("{} is: {} bytes long", display, buffer.len()),
}

// let mut offset = 0;
// for chunk in buffer.chunks(16) {
//     print!("{:07x} ", offset);
//     for b in chunk {
//         print!("{:02x} ", b);
//     }
//     offset += 16;
//     println!("");
// }
// println!("{:07x} ", offset);

