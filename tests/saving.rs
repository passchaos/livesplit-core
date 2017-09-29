extern crate livesplit_core;

mod save {
    use std::fs::File;
    use std::io::BufReader;
    use livesplit_core::Run;
    use livesplit_core::run::parser::livesplit;
    use livesplit_core::run::saver::{file_write, wsplit};

    fn file(path: &str) -> BufReader<File> {
        BufReader::new(File::open(path).unwrap())
    }

    fn livesplit(path: &str) -> Run {
        livesplit::parse(file(path), None).unwrap()
    }

    #[test]
    fn wsplit() {
        let run = livesplit("tests/run_files/livesplit1.6_gametime.lss");
        let mut vec = Vec::new();
        wsplit::save(&run, file_write::OnlyStoreMainFile(&mut vec)).unwrap();
        let got = String::from_utf8(vec).unwrap();
        let expected = r#"Title=Dishonored - Any%
Attempts=1958
Offset=0
Size=152,25
WatchYr<3Die,0,191.265,5.432
Elevator Land,0,264.86,2.003
Gross, Rats,0,395.925,131.38
Try Not Falling?,0,628.423,231.735
Men of the Faith,0,775.708,146.673
Fuckin Celia,0,894.514,112.506
Tr4p Haus,0,1066.217,165.834
Troubled Waters,0,1371.376,160.397
RNG Roulette,0,1586.956,210.858
Not At Fault,0,1833.368,236.252
Amateur Work,0,2211.204,378.561
Don't Mind Me,0,2278.29,61.884
H9 U Havelock,0,2401.517,122.007
Icons="","","","","","","","","","","","",""
"#;
        assert_eq!(expected, got);
    }
}
