use clap::Clap;
use digest::Digest;

#[derive(Clap)]
struct Opts {
    path: std::path::PathBuf,
}

fn main() {
    let opts: Opts = Opts::parse();
    let mut hasher = blake3::Hasher::new();

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
            match sync::FileHash::new(result.path(), &mut hasher) {
                Ok(file_hash) => contents.add_file(path, file_hash),
                Err(error) => println!("Error: {:?}", error),
            }
        } else {
            contents.add_dir(path);
        }
    }

    println!("{}", serde_yaml::to_string(&contents).unwrap());
}
