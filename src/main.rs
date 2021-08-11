use clap::Clap;
use sha2::Digest;

#[derive(Clap)]
struct Opts {
    path: std::path::PathBuf,
}

fn main() {
    let opts: Opts = Opts::parse();
    let mut buffer = vec![0; 32 * 1024];
    let buffer: &mut [u8] = &mut buffer;
    let mut hasher = sha2::Sha384::new();

    let mut contents = sync::Contents::default();
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
            match sync::FileHash::new(result.path(), buffer, &mut hasher) {
                Ok(file_hash) => contents.add_file(path, file_hash),
                Err(error) => println!("Error: {:?}", error),
            }
        } else {
            contents.add_dir(path);
        }
    }

    println!("{}", serde_yaml::to_string(&contents).unwrap());
}
