use std::path::PathBuf;

use structopt::StructOpt;

mod ktsl2stbin;
use ktsl2stbin::Ktsl2stbin;

mod ktsl2asbin;
use ktsl2asbin::Ktsl2asbin;

mod ktsl;
pub use ktsl::Ktsl;

mod sections;
pub use sections::*;

use binread::Error::*;

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
    #[structopt(parse(from_os_str))]
    asbin_path: Option<PathBuf>
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
        Command::Print(args) => {
            let ktsl = match Ktsl2stbin::open(&args.path) {
                Ok(content) => content,
                // TODO: Handle this better
                Err(_) => panic!("Error while trying to open {}", &args.path.display()),
            };

            println!("Game ID: {}\nCompressed: {}\nDecompressed size: 0x{:08x}\nKTSL count: {}", ktsl.header.game_id, ktsl.header.comp_size != ktsl.header.decomp_size,ktsl.header.decomp_size, ktsl.entries.len());
        },
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
        Command::Pack(args) => {
            let mut ktsl = Ktsl2stbin::new();
            // TODO: Ask for GameID or figure it out somehow
            //ktsl.pack(&args.path);

            // Extreme grossness
            let asbin = match args.asbin_path {
                Some(asbin_path) => {
                    let asbin = match Ktsl2asbin::open(&asbin_path) {
                        // 'Sup Devin?
                        Ok(rooster) => {
                            Some(Box::new(rooster))
                        },
                        Err(_) => None,
                    };
                    asbin
                },
                None => None,
            };
            

            ktsl.pack(&args.path, asbin);
        },
        _ => { println!("Unimplemented"); },
    }
}

mod tests {
    use core::panic;

    use super::*;
    
    #[test]
    fn test() {
        let ktsl: Ktsl2stbin = Ktsl2stbin::open("./BGM_DLC_EN.ktsl2stbin").unwrap();
    }

    #[test]
    fn test_ktsl_stbin_parse() {
        let ktsl: Ktsl = Ktsl::open("./BGM_DLC_EN.ktsl2stbin").unwrap();
    }

    #[test]
    fn test_ktsl_asbin_parse() {
        let ktsl: Ktsl = Ktsl::open("./31011.ktsl2asbin").unwrap();
        dbg!(ktsl);
    }

    #[test]
    fn test_asbin() {
        let mut ktsl: Ktsl2asbin = match Ktsl2asbin::open("./31011.ktsl2asbin") {
            Ok(ktsl) => ktsl,
            Err(err) => match err {
                binread::Error::EnumErrors { pos, variant_errors } => panic!("Pos: {}\nErrors: {:?}", pos, variant_errors),
                _ => panic!(),
            },
        };

        let test: Vec<&mut ktsl2asbin::Section> = ktsl.entries.iter_mut().filter_map(|section| {
            if let ktsl2asbin::Section::Adpcm(_) = section {
                return Some(section)
            }

            None
        }).collect();
        
        dbg!(test.len());
    }
}
