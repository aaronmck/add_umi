extern crate rust_htslib;
extern crate clap;
extern crate regex;

#[macro_use]
extern crate lazy_static;

use regex::bytes::Regex;

use clap::{Arg, App};
use rust_htslib::bam;
use rust_htslib::bam::Read;

/// read in a bam file, parse out the UMI, and write the UMI into the expected header tag
fn main() {

    // load up the matchers
    let matches = App::new("AddUMI")
        .version("1.0")
        .author("Aaron McKenna <aaronatwpi@gmail.com>")
        .about("Given a BAM file with a UMI specified in the name of the read, move it to the UMI tag within the read")
        .arg(Arg::with_name("inBAM")
             .short("i")
             .long("inBAM")
             .value_name("FILE")
             .help("the input bam")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("outBAM")
             .short("o")
             .long("outBAM")
             .value_name("FILE")
             .help("the output bam")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("tag")
             .help("the tag which to extract from the read name, and add as a tag on the bam file")
             .short("t")
             .long("tag")
             .value_name("STRING")
             .takes_value(true)
             .required(true))
        .get_matches();

    // setup the input and output files, plus tag
    let input_bam = matches.value_of("inBAM").unwrap_or("failed.bam");
    println!("Value for input bam: {}", input_bam);

    let output_bam = matches.value_of("outBAM").unwrap_or("failed.bam");
    println!("Value for output bam: {}", output_bam);

    let input_tag = matches.value_of("tag").unwrap_or("SL.umi.read1.0.8");
    println!("Value for output tag: {}", input_tag);

    // setup the input and output BAM reader/writers
    let mut bam = bam::Reader::from_path(input_bam).unwrap();
    let header = bam::Header::from_template(bam.header());
    let mut out = bam::Writer::from_path(output_bam, &header).unwrap();

    // count our output warnings
    let mut cnt = 0;
    
    // copy reverse reads to new BAM file
    for r in bam.records() {
        let mut record = r.unwrap();

        // println!("{}", show(record.qname()));
        let read_umi = extract_read_umi(record.qname(), input_tag.as_bytes());
        if read_umi.is_some() {
            record.push_aux(b"RX",&bam::record::Aux::String(&read_umi.unwrap()[..]));
        } else {
            if cnt < 100 {
                println!("Unable to parse out read name from {}",show(record.qname()));
                cnt += 1;
            }
        };
        
        out.write(&record).unwrap();
    }
}

/// given a read name with a very particular naming scheme, extract the UMI
/// sequence. 
fn extract_read_umi(read_name: &[u8], id: &[u8]) -> Option<Vec<u8>> {
    lazy_static! {
        // create a regex for the tags within a read name
        static ref RE: Regex = Regex::new(r"\{([a-zA-Z_0-9.]+)\}=\{(\w+)\},\{(.+?)\};").unwrap();
    }
    
    RE.captures_iter(read_name).filter_map(|cap| {
        let groups = (cap.get(1), cap.get(2), cap.get(3));
        // if we want output -- println!("{} -> {},{}",show(cap.get(1).unwrap().as_bytes()), show(cap.get(2).unwrap().as_bytes()), show(cap.get(3).unwrap().as_bytes()));
        match groups {
            (Some(ref key), Some(value), Some(_quals)) if key.as_bytes() == id => Some(value.as_bytes().to_vec()),
            (Some(_),Some(_),Some(_)) => None,
            _ => None,
        }
    }).next()
}

/// a simple function to try and convert a u8 slice to a string. This won't
/// handle special characters with any grace.
/// from https://stackoverflow.com/questions/41449708/how-to-print-a-u8-slice-as-text-if-i-dont-care-about-the-particular-encoding
fn show(bs: &[u8]) -> String {
    String::from_utf8_lossy(bs).into_owned()
}
