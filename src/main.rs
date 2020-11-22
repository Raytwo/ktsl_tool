use std::path::PathBuf;

use structopt::StructOpt;

mod ktsl2stbin;
use ktsl2stbin::Ktsl2stbin;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "KtslTool",
    about = "Simple command-line tool to manipulate KTSL (Koei Tecmo Sound Library) files."
)]
struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Reserved
    Extract,
    /// Reserved
    Inject,
    /// Unpacks a KTSL archive to a directory with the proper file hierarchy for repacking
    Unpack(Unpack),
    /// Packs a directory into a KTSL archive using directory names
    Pack(Pack),
    /// Output relevant informations about a KTSL archive
    Print(Print),
}

// TODO: Turn all the reused args into a separate struct?

#[derive(Debug, StructOpt)]
struct Print {
    /// Decompressed the file
    #[structopt(short = "gz", long = "gzip")]
    gz: bool,
    /// Path to the file to print
    #[structopt(parse(from_os_str))]
    path: PathBuf
}

#[derive(Debug, StructOpt)]
struct Pack {
    /// Compress the file
    #[structopt(short = "gz", long = "gzip")]
    gz: bool,
    /// Path to the directory to pack
    #[structopt(parse(from_os_str))]
    path: PathBuf,
}

#[derive(Debug, StructOpt)]
struct Unpack {
    /// Decompress the file
    #[structopt(short = "gz", long = "gzip")]
    gz: bool,
    /// Path to the file to unpack (Ktsl2stbin only)
    #[structopt(parse(from_os_str))]
    path: PathBuf,
    /// Directory where the files are to be extracted. Defaults to "./out".
    #[structopt(parse(from_os_str), default_value("./out"))]
    out_dir: PathBuf,
}

fn main() {
    let opt = Args::from_args();

    match opt.cmd {
        Command::Unpack(args) => {
            let ktsl = match Ktsl2stbin::open(&args.path) {
                Ok(content) => content,
                // TODO: Handle this better
                Err(_) => panic!("Error while trying to open {}", &args.path.display()),
            };

            // Create directory and childs just in case
            std::fs::create_dir_all(&args.out_dir).unwrap();

            // Unpack KTSR content in there
            ktsl.unpack(&args.out_dir);
        },
        _ => { println!("Unimplemented"); },
    }
}

mod tests {
    use super::*;
    
    #[test]
    fn test() {
        let ktsl: Ktsl2stbin = Ktsl2stbin::open("./BGM_DLC_EN.ktsl2stbin").unwrap();
    }
}
