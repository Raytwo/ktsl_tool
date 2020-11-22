use std::path::PathBuf;

use binread::{io::Cursor, BinRead};

use structopt::StructOpt;

mod ktsl2stbin;
use ktsl2stbin::Ktsl2stbin;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "KtslTool",
    about = "Simple command-line tool to manipulate KTSL (Koei Tecmo Sound Library) files."
)]
enum Args {
    /// Reserved
    Extract {
    },
    /// Reserved
    Inject {
    },
    /// Unpacks a KTSL archive to a directory with the proper file hierarchy for repacking
    Unpack {
        #[structopt(short = "gz", long = "gzip", help = "Uncompresses the file")]
        gz: bool,
        #[structopt(parse(from_os_str), help = "Path to the file")]
        path: PathBuf,
    },
    /// Packs a directory into a KTSL archive using directory names
    Pack {
        #[structopt(short = "gz", long = "gzip", help = "Compresses the file")]
        gz: bool,
        #[structopt(parse(from_os_str), help = "Path to the directory to pack")]
        path: PathBuf
    },
    /// Output relevant informations about a KTSL archive
    Print {
        #[structopt(short = "gz", long = "gzip", help = "Uncompresses the file")]
        gz: bool,
        #[structopt(parse(from_os_str), help = "Path to the file to print")]
        path: PathBuf
    }
}


fn main() {
    let args = Args::from_args();

    let file = std::fs::File::open("./BGM_DLC_EN.ktsl2stbin").unwrap();
    let mut reader = std::io::BufReader::new(file);
    let file: Box<Ktsl2stbin> = Box::new(Ktsl2stbin::read(&mut reader).unwrap());
    file.unpack();

    match args {
        Print => {
            println!("lmao no");
        },
        _ => { println!("lmao also no"); },
    }
}

mod tests {
    use super::*;

    //const TEST_CONTENTS: &[u8] = include_bytes!("../BGM_DLC_EN.ktsl2stbin");

    #[test]
    fn test() {
        let file: Ktsl2stbin = Ktsl2stbin::read(&mut Cursor::new(&std::fs::read("./BGM_DLC_EN.ktsl2stbin").unwrap())).unwrap();
        file.unpack();
        //dbg!(&file.entries[0]);
    }
}
