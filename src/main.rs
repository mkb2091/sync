use clap::Clap;
use sha2::Digest;
use std::io::Read;

#[derive(Clap)]
struct Opts {
    path: std::path::PathBuf,
}

fn main() {
    let opts: Opts = Opts::parse();
    let mut buffer = vec![0; 32 * 1024];
    let buffer: &mut [u8] = &mut buffer;
    let mut hasher = sha2::Sha384::new();
    for result in ignore::WalkBuilder::new(&opts.path)
        .build()
        .filter_map(|x| x.ok())
    {
        let path = if let Ok(path) = result.path().strip_prefix(&opts.path) {
            path
        } else {
            continue;
        };
        if result
            .metadata()
            .map(|metadata| metadata.is_file())
            .unwrap_or(true)
        {
            match std::fs::File::open(result.path()) {
                Ok(mut file) => {
                    let mut counter = 0;
                    loop {
                        match file.read(buffer) {
                            Ok(n) => {
                                if n == 0 {
                                    break;
                                }
                                hasher.update(&buffer[..std::cmp::max(n, buffer.len())]);
                            }
                            Err(error) => {
                                println!("Error: {:?}", error);
                                break;
                            }
                        }
                        counter += 1;
                    }
                    println!("Path: {:?}", path);
                    println!("Hash: {:?}", hasher.finalize_reset());
                    println!("Counter: {:?}", counter);
                }

                Err(error) => println!("Error: {:?}", error),
            }
        } else {
            println!("Dir: {:?}", path);
        }
    }
}
